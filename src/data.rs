use bincode::{serialize_into, deserialize_from};
use failure::Error;
use git2::{
    build::{CheckoutBuilder, RepoBuilder}, Cred, FetchOptions, IndexAddOption, PushOptions, RemoteCallbacks,
    Repository, Signature,
};
use std::{
    fs::{create_dir, remove_dir_all, File},
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use super::core::astronomicals::Galaxy;
use super::core::economy::Economy;
use super::player::Player;
use super::simulate::resources::{fetch_resource, ShipResource};
use super::core::ship::Shipyard;
use super::config::Data as DataConfig;
use super::core::game::Game;

/// Data service for interfacing with the game data on local and remote locations.
pub struct DataService {
    config: DataConfig,
    repo: Repository,
}

impl<'a> DataService {
    /// Create a new data service, loading and syncing all game data.
    pub fn new(config: DataConfig) -> Result<DataService, Error> {
        let auth =
            Self::create_auth_callback(config.public_key.clone(), config.private_key.clone());
        let repo = Self::open_repository(&config.local, &config.remote, auth)?;
        Ok(DataService { config, repo })
    }

    /// Wrapper for creating Auth type.
    fn auth(&self) -> RemoteCallbacks<'a> {
        Self::create_auth_callback(
            self.config.public_key.clone(),
            self.config.private_key.clone(),
        )
    }

    /// Create a remote callback for authentication needed for libgit.
    fn create_auth_callback(public_key: PathBuf, private_key: PathBuf) -> RemoteCallbacks<'a> {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(move |_, _, _| {
            Cred::ssh_key("git", Some(&public_key), &private_key, None)
        });
        callbacks
    }

    /// Opens the given remote repository in the local path, if already exists, 
    /// any remote changes will be synced down.
    fn open_repository(
        local: &PathBuf,
        remote: &str,
        auth: RemoteCallbacks,
    ) -> Result<Repository, Error> {
        let mut options = FetchOptions::new();
        options.remote_callbacks(auth);

        // Try opening repository if it already exits.
        match Repository::open(local) {
            Ok(repo) => {
                debug!("Repository already exists, opening");
                debug!("Fetching remote data if needed");
                debug!("Current head: {}", repo.head()?.peel_to_commit()?.id());
                repo.find_remote("origin")?.fetch(&["master"], Some(&mut options), None)?;
                // Dummy block to avoid NLL.
                {
                    let remote_ref = repo.find_reference("refs/remotes/origin/master")?;
                    let remote_commit = repo.reference_to_annotated_commit(&remote_ref)?;
                    debug!("Remote head: {}", remote_commit.id());
                    let mut builder = CheckoutBuilder::new();
                    builder.force();
                    repo.checkout_head(Some(&mut builder))?;
                }
                Ok(repo)
            }
            Err(_) => {
                // Attempt to clean data directory, failure is ok.
                let _ = remove_dir_all(local);
                create_dir(local)?;

                // Clone the simulatino data repository.
                debug!("Cloning repository: {:?} to {:?}...", remote, local);
                let repo = RepoBuilder::new()
                    .fetch_options(options).clone(remote, local)?;
                repo.remote_add_push("origin", "refs/heads/master:refs/heads/master")?;
                debug!("Cloning sucessful");
                Ok(repo)
            }
        }
    }

    /// Syncs all changes upstream in the given repository, using the given message to note the change.
    pub fn sync_up(&self, message: &str) -> Result<(), Error> {
        // Update index and commit
        let mut index = self.repo.index()?;
        index.add_all(["*"].iter(), IndexAddOption::FORCE, None)?;
        let oid = index.write_tree()?;
        let signature = Signature::now("SimBot", "SimBot")?;
        let tree = self.repo.find_tree(oid)?;
        self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &message,
            &tree,
            &[&self.repo.head()?.peel_to_commit()?],
        )?;

        debug!("Syncing data upstream");
        // Data to the remote.
        // TODO: Maybe read branch from config.
        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(self.auth());
        self.repo
            .find_remote("origin")
            .and_then(|ref mut remote| remote.push(&[], Some(&mut push_options)))?;

        Ok(())
    }

    /// Store the game data, syncing with the remote.
    /// TODO: Fails if directory does not exist, repository is not cloned.
    pub fn store(&self, game: &Game) -> Result<(), Error> {
        let base_path = &self.config.local;

        // Store all data on disk
        let mut galaxy_file =
            BufWriter::new(File::create(base_path.join("galaxy.cbor").as_path())?);
        serialize_into(&mut galaxy_file, &game.galaxy)?;
        let mut player_file =
            BufWriter::new(File::create(base_path.join("player.cbor").as_path())?);
        serialize_into(&mut player_file, &game.player)?;
        let mut economy_file =
            BufWriter::new(File::create(base_path.join("economy.cbor").as_path())?);
        serialize_into(&mut economy_file, &game.economy)?;
        let mut update_file =
            BufWriter::new(File::create(base_path.join("updated.cbor").as_path())?);
        serialize_into(&mut update_file, &game.updated)?;

        Ok(())
    }

    /// Attempts to load game data from disk.
    pub fn try_load(&self) -> Result<Game, Error> {
        let base_path = &self.config.local;

        // Load all data from disk.
        let mut galaxy_file =
            BufReader::new(File::open(base_path.join("galaxy.cbor").as_path())?);
        let galaxy = deserialize_from(&mut galaxy_file)?;
        let mut player_file =
            BufReader::new(File::open(base_path.join("player.cbor").as_path())?);
        let player = deserialize_from(&mut player_file)?;
        let mut economy_file =
            BufReader::new(File::open(base_path.join("economy.cbor").as_path())?);
        let economy = deserialize_from(&mut economy_file)?;
        let mut update_file =
            BufReader::new(File::open(base_path.join("updated.cbor").as_path())?);
        let updated = deserialize_from(&mut update_file)?;

        let mut shipyard = Shipyard::new();
        shipyard.add_ships(fetch_resource::<ShipResource>().unwrap());

        Ok(Game {
            galaxy,
            shipyard,
            player,
            economy,
            updated
        })
    }
}
