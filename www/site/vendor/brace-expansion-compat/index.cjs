"use strict";

const implementation = require("brace-expansion-v5");

// minimatch 3 expects the historical callable CommonJS export, while
// minimatch 10 consumes the modern named `expand` export. This adapter keeps
// both interfaces on the patched brace-expansion 5 implementation.
module.exports = implementation.expand;
module.exports.expand = implementation.expand;
module.exports.EXPANSION_MAX = implementation.EXPANSION_MAX;
module.exports.EXPANSION_MAX_LENGTH = implementation.EXPANSION_MAX_LENGTH;
