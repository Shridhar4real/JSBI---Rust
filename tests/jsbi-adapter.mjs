import { execFileSync } from 'child_process';
import { fileURLToPath } from 'url';
import path from 'path';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

function getBinaryPath() {
  const isWin = process.platform === 'win32';
  const binName = isWin ? 'jsbi-cli.exe' : 'jsbi-cli';
  const releasePath = path.join(projectRoot, 'target', 'release', binName);
  const debugPath = path.join(projectRoot, 'target', 'debug', binName);

  if (fs.existsSync(releasePath)) return releasePath;
  if (fs.existsSync(debugPath)) return debugPath;
  throw new Error(`jsbi-cli executable not found at ${releasePath} or ${debugPath}. Run 'cargo build --release' first.`);
}

const BINARY_PATH = getBinaryPath();

function execRust(op, aStr, bStr = '0') {
  try {
    const out = execFileSync(BINARY_PATH, ['eval', op, String(aStr), String(bStr)], {
      encoding: 'utf-8',
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    return out.trim();
  } catch (err) {
    const stderr = err.stderr ? err.stderr.toString() : err.message;
    if (stderr.includes('SyntaxError') || stderr.includes('Cannot convert')) {
      throw new SyntaxError(stderr.trim());
    }
    if (stderr.includes('Division by zero') || stderr.includes('Exponent must be positive') || stderr.includes('RangeError')) {
      throw new RangeError(stderr.trim());
    }
    throw new Error(stderr.trim());
  }
}

export default class JSBI {
  constructor(strVal) {
    this.__val = String(strVal);
  }

  toString(radix = 10) {
    if (radix === 10) {
      return this.__val;
    }
    return execRust('toString', this.__val, String(radix));
  }

  valueOf() {
    throw new Error('Convert JSBI instances to native numbers using `toNumber`.');
  }

  [Symbol.toPrimitive](hint) {
    if (hint === 'number') {
      throw new Error('Convert JSBI instances to native numbers using `toNumber`.');
    }
    return this.__val;
  }

  static BigInt(val) {
    if (val instanceof JSBI) return val;
    if (val === null || val === undefined) {
      throw new TypeError(`Cannot convert ${val} to a BigInt`);
    }

    let coerced = val;
    if (typeof coerced === 'object') {
      if (typeof coerced[Symbol.toPrimitive] === 'function') {
        coerced = coerced[Symbol.toPrimitive]('number');
      } else if (typeof coerced.valueOf === 'function') {
        coerced = coerced.valueOf();
      }
    }

    if (coerced instanceof JSBI) return coerced;

    if (typeof coerced === 'boolean') {
      coerced = coerced ? 1 : 0;
    }

    if (typeof coerced === 'number') {
      if (!Number.isFinite(coerced) || coerced !== Math.floor(coerced)) {
        throw new RangeError(`The number ${coerced} cannot be converted to a BigInt because it is not an integer`);
      }
      const normStr = execRust('BigInt', String(coerced));
      return new JSBI(normStr);
    }

    if (typeof coerced === 'string') {
      const normStr = execRust('BigInt', coerced);
      return new JSBI(normStr);
    }

    const normStr = execRust('BigInt', String(coerced));
    return new JSBI(normStr);
  }

  static add(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('add', aVal, bVal));
  }

  static subtract(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('subtract', aVal, bVal));
  }
  static sub(a, b) { return JSBI.subtract(a, b); }

  static multiply(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('multiply', aVal, bVal));
  }
  static mul(a, b) { return JSBI.multiply(a, b); }

  static divide(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('divide', aVal, bVal));
  }
  static div(a, b) { return JSBI.divide(a, b); }

  static remainder(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('remainder', aVal, bVal));
  }
  static rem(a, b) { return JSBI.remainder(a, b); }
  static mod(a, b) { return JSBI.remainder(a, b); }

  static exponentiate(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('exponentiate', aVal, bVal));
  }
  static exp(a, b) { return JSBI.exponentiate(a, b); }

  static unaryMinus(a) {
    const aVal = JSBI.BigInt(a).__val;
    return new JSBI(execRust('unaryMinus', aVal));
  }
  static neg(a) { return JSBI.unaryMinus(a); }

  static bitwiseAnd(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('bitwiseAnd', aVal, bVal));
  }
  static and(a, b) { return JSBI.bitwiseAnd(a, b); }

  static bitwiseOr(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('bitwiseOr', aVal, bVal));
  }
  static or(a, b) { return JSBI.bitwiseOr(a, b); }

  static bitwiseXor(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('bitwiseXor', aVal, bVal));
  }
  static xor(a, b) { return JSBI.bitwiseXor(a, b); }

  static bitwiseNot(a) {
    const aVal = JSBI.BigInt(a).__val;
    return new JSBI(execRust('bitwiseNot', aVal));
  }
  static not(a) { return JSBI.bitwiseNot(a); }

  static leftShift(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('leftShift', aVal, bVal));
  }
  static shl(a, b) { return JSBI.leftShift(a, b); }

  static signedRightShift(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return new JSBI(execRust('signedRightShift', aVal, bVal));
  }
  static sar(a, b) { return JSBI.signedRightShift(a, b); }

  static equal(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return execRust('equal', aVal, bVal) === 'true';
  }
  static EQ(a, b) { return JSBI.equal(a, b); }

  static notEqual(a, b) {
    return !JSBI.equal(a, b);
  }
  static NE(a, b) { return JSBI.notEqual(a, b); }

  static lessThan(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return execRust('lessThan', aVal, bVal) === 'true';
  }
  static LT(a, b) { return JSBI.lessThan(a, b); }

  static lessThanOrEqual(a, b) {
    return !JSBI.greaterThan(a, b);
  }
  static LE(a, b) { return JSBI.lessThanOrEqual(a, b); }

  static greaterThan(a, b) {
    const aVal = JSBI.BigInt(a).__val;
    const bVal = JSBI.BigInt(b).__val;
    return execRust('greaterThan', aVal, bVal) === 'true';
  }
  static GT(a, b) { return JSBI.greaterThan(a, b); }

  static greaterThanOrEqual(a, b) {
    return !JSBI.lessThan(a, b);
  }
  static GE(a, b) { return JSBI.greaterThanOrEqual(a, b); }

  static asIntN(bits, a) {
    const aVal = JSBI.BigInt(a).__val;
    return new JSBI(execRust('asIntN', aVal, String(bits)));
  }

  static asUintN(bits, a) {
    const aVal = JSBI.BigInt(a).__val;
    return new JSBI(execRust('asUintN', aVal, String(bits)));
  }

  static toNumber(a) {
    const aVal = JSBI.BigInt(a).__val;
    return Number(execRust('toNumber', aVal));
  }

  static DataViewSetBigInt64(dataview, byteOffset, value, littleEndian = false) {
    const jsb = JSBI.asIntN(64, value);
    const b = BigInt(jsb.toString());
    dataview.setBigInt64(byteOffset, b, littleEndian);
  }

  static DataViewGetBigInt64(dataview, byteOffset, littleEndian = false) {
    const b = dataview.getBigInt64(byteOffset, littleEndian);
    return JSBI.BigInt(b.toString());
  }

  static DataViewSetBigUint64(dataview, byteOffset, value, littleEndian = false) {
    const jsb = JSBI.asUintN(64, value);
    const b = BigInt(jsb.toString());
    dataview.setBigUint64(byteOffset, b, littleEndian);
  }

  static DataViewGetBigUint64(dataview, byteOffset, littleEndian = false) {
    const b = dataview.getBigUint64(byteOffset, littleEndian);
    return JSBI.BigInt(b.toString());
  }
}
