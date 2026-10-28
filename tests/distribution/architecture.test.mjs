import test from 'node:test';
import assert from 'node:assert/strict';
import {dependencyErrors, sourceErrors} from '../../scripts/dev/rules.mjs';

test('le cœur ne peut pas acquérir un adaptateur ou un framework, même en dépendance de test', () => {
  const packages = [
    {name: 'portloom-domain', dependencies: [{name: 'regex'}]},
    {name: 'portloom-application', dependencies: [{name: 'portloom-domain'}]},
  ];
  assert.deepEqual(dependencyErrors(packages), []);
  packages[1].dependencies.push({name: 'ratatui', kind: 'dev'});
  assert.match(dependencyErrors(packages).join(), /portloom-application.*ratatui/);
  assert.match(dependencyErrors([]).join(), /Crate manquante/);
});

test('les accès système directs et regroupés sont refusés dans les crates métier', () => {
  for (const source of ['use std::fs;', 'use std::{sync::Arc, fs::{self, File}};', 'std::env::var("HOME");', 'use std::time::Instant;']) {
    assert.ok(sourceErrors('crates/application/src/stop.rs', source).length, source);
  }
  assert.deepEqual(sourceErrors('crates/domain/src/model.rs', 'use std::{collections::BTreeMap, sync::Arc};'), []);
});

test('la présentation et les adaptateurs restent indépendants dans les deux sens', () => {
  assert.ok(sourceErrors('src/presentation/view.rs', 'use crate::{domain::model, infrastructure::system};').length);
  assert.ok(sourceErrors('src/infrastructure/demo.rs', 'crate::presentation::State::new();').length);
  assert.deepEqual(sourceErrors('src/presentation/view.rs', 'use crate::{application::ports, domain::model};'), []);
  assert.deepEqual(sourceErrors('src/composition/mod.rs', 'use crate::{infrastructure, presentation};'), []);
});
