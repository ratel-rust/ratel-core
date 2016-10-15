const path = require('path');
const assert = require('assert');
const basePath = path.join.bind(path, __dirname, '..');

global.RUST_BACKTRACE = 'RUST_BACKTRACE' in process.env;

global.assert = assert;
global.basePath = basePath;
global.path = path;
global.requireRelativePath = (...path) => require(global.basePath(...path));

global.Ratel = global.requireRelativePath('.');
