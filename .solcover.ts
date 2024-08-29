module.exports = {
  skipFiles: ['test', 'interfaces'],
  istanbulReporter: ['text', 'text-summary'],
  modifierWhitelist: ['initializer', 'onlyInitializing', 'reinitializer'],
  mocha: {
    grep: "gas",
    invert: true
  }
};