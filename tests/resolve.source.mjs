/*
 ** Copyright (C) 2018-2019 Bloomberg LP. All rights reserved.
 ** This code is governed by the license found in the LICENSE file.
 */

import fs from 'fs';
const PKG = JSON.parse(fs.readFileSync('package.json', {encoding: 'utf-8'}));

export function resolve(specifier, parent, defaultResolve) {
  if (
    specifier === PKG.name ||
    specifier.includes('jsbi') ||
    specifier.includes('dist/jsbi') ||
    specifier.includes('tsc-out/jsbi')
  ) {
    specifier = new URL('./jsbi-adapter.mjs', import.meta.url).toString();
    return {
      shortCircuit: true,
      url: specifier,
    };
  }
  return defaultResolve(specifier, parent);
}
