from __future__ import division
import random
import sys


class Categorical(object):

    def __init__(self, support, prior):
        self.counts = {x: prior for x in support}
        self.total = sum(self.counts.itervalues())

    def observe(self, event, count=1):
        self.counts[event] += count
        self.total += count

    def sample(self, dice=random):
        sample = dice.uniform(0, self.total)
        for event, count in self.counts.iteritems():
            if sample <= count:
                return event
            sample -= count

    def __getitem__(self, event):
        return self.counts[event] / self.total


class MarkovModel(object):

    def __init__(self, support, order, prior, boundary_symbol=None):
        self.support = set(support)
        self.support.add(boundary_symbol)
        self.order = order
        self.prior = prior
        self.boundary = boundary_symbol
        self.prefix = [self.boundary] * self.order
        self.postfix = [self.boundary]
        self.counts = {}

    def _categorical(self, context):
        if context not in self.counts:
            self.counts[context] = Categorical(self.support, self.prior)
        return self.counts[context]

    def _backoff(self, context):
        context = tuple(context)
        if len(context) > self.order:
            context = context[-self.order:]
        elif len(context) < self.order:
            context = (self.boundary,) * (self.order - len(context)) + context

        while context not in self.counts and len(context) > 0:
            context = context[1:]
        return context

    def observe(self, sequence, count=1):
        sequence = self.prefix + list(sequence) + self.postfix
        for i in range(self.order, len(sequence)):
            context = tuple(sequence[i - self.order:i])
            event = sequence[i]
            for j in range(len(context) + 1):
                self._categorical(context[j:]).observe(event, count)

    def sample(self, context):
        context = self._backoff(context)
        return self._categorical(context).sample()

    def generate(self):
        sequence = [self.sample(self.prefix)]
        while sequence[-1] != self.boundary:
            sequence.append(self.sample(sequence))
        return sequence[:-1]

    def __getitem__(self, condition):
        event = condition.start
        context = self._backoff(condition.stop)
        return self._categorial(context)[event]

class NameGenerator(object):

    def __init__(self, name_file, order=3, prior=.001):
        self.names = set()
        support = set()
        for name in name_file:
            name = name.strip()
            if len(name) > 0:
                self.names.add(name)
                support.update(name)
        self.model = MarkovModel(support, order, prior)
        for name in self.names:
            self.model.observe(name)

    def generate(self):
        name = ''.join(self.model.generate())
        while name in self.names or len(name) > 8 or len(name) < 4:
            name = ''.join(self.model.generate())
        self.names.add(name)
        return name

if __name__ == "__main__":
    tries = sys.argv[1]

    files = ["eso.txt", "russian.txt", "greek.txt", "french.txt", "sw2.txt"];

    for file_name in files:
        file = open(file_name, "r")
        gen = NameGenerator(file)

        for i in range(int(tries)):
            print gen.generate()
