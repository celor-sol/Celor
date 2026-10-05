import test from 'node:test';
import assert from 'node:assert/strict';
import { SlotClock } from '../lib/chrono-core/slot-clock.ts';
import { BankGraph } from '../lib/chrono-core/bank-graph.ts';
import { LeaderEngine } from '../lib/chrono-core/leader-engine.ts';
import { FinalityEngine } from '../lib/chrono-core/finality-engine.ts';
import { ChronoEventBus } from '../lib/chrono-core/event-bus.ts';

test('SlotClock initializes and computes dynamic slot progress accurately', () => {
  const clock = new SlotClock(250);
  assert.equal(clock.getTargetSlotDuration(), 250);

  clock.onSlotObserved(1000);
  assert.equal(clock.getCurrentSlot(), 1000);

  const progress = clock.getSlotProgress();
  assert.equal(progress.slot, 1000);
  assert.equal(progress.targetSlotDurationMs, 250);
  assert.equal(progress.isRealTime, true);
  assert.ok(progress.phaseRatio >= 0 && progress.phaseRatio <= 1.0);
});

test('BankGraph manages candidate banks and handles UpdateParent', () => {
  const bankGraph = new BankGraph();
  const slot = 5000;

  bankGraph.registerBank({
    bankId: 'BANK 01',
    slot,
    parentBankId: 'bank-4999',
    blockhash: 'Hash01',
    state: 'OBSERVED',
    txCount: 42,
    observedAtMs: 1000,
  });

  bankGraph.registerBank({
    bankId: 'BANK 02',
    slot,
    parentBankId: 'bank-4999',
    blockhash: 'Hash02',
    state: 'OBSERVED',
    txCount: 55,
    observedAtMs: 1050,
  });

  let banks = bankGraph.getBanksForSlot(slot);
  assert.equal(banks.length, 2);

  // Trigger UpdateParent
  const parentSwitch = bankGraph.handleUpdateParent(slot, 'BANK 01', 'BANK 02', 'Test parent switch');
  assert.equal(parentSwitch.slot, slot);
  assert.equal(parentSwitch.abandonedBankId, 'BANK 01');
  assert.equal(parentSwitch.canonicalBankId, 'BANK 02');

  banks = bankGraph.getBanksForSlot(slot);
  const bank01 = banks.find((b) => b.bankId === 'BANK 01');
  const bank02 = banks.find((b) => b.bankId === 'BANK 02');

  assert.equal(bank01.state, 'ABANDONED');
  assert.equal(bank02.state, 'CANONICAL');
});

test('LeaderEngine tracks slot schedule and fast leader handoff phases', () => {
  const engine = new LeaderEngine();
  engine.updateLeaderForSlot(200, 'LeaderA');
  engine.updateLeaderForSlot(201, 'LeaderB');
  engine.updateLeaderForSlot(202, 'LeaderC');

  const buildingInfo = engine.recomputeLeaders(200, 0.2);
  assert.equal(buildingInfo.currentLeader, 'LeaderA');
  assert.equal(buildingInfo.nextLeader, 'LeaderB');
  assert.equal(buildingInfo.handoffState, 'BUILDING');

  const readyInfo = engine.recomputeLeaders(200, 0.7);
  assert.equal(readyInfo.handoffState, 'PARENT_READY');

  const handoverInfo = engine.recomputeLeaders(200, 0.95);
  assert.equal(handoverInfo.handoffState, 'HANDOVER');
});

test('FinalityEngine tracks Observed -> Canonical -> Finalized lifecycle', () => {
  const engine = new FinalityEngine();
  const slot = 3000;

  engine.onSlotObserved(slot, 1000);
  let info = engine.getFinalityForSlot(slot);
  assert.equal(info.status, 'OBSERVED');

  engine.onSlotCanonical(slot, 1080);
  info = engine.getFinalityForSlot(slot);
  assert.equal(info.status, 'CANONICAL');

  engine.onSlotFinalized(slot, 'BLS_FAST_PATH_CERT', 88.5, 1140);
  info = engine.getFinalityForSlot(slot);
  assert.equal(info.status, 'FINALIZED');
  assert.equal(info.isFastPath, true);
  assert.equal(info.finalityLatencyMs, 140);
});

test('ChronoEventBus dispatches typed events without cross-contamination', () => {
  const bus = new ChronoEventBus();
  let slotReceived = false;
  let leaderReceived = false;

  const unsubSlot = bus.onSlot((p) => {
    slotReceived = true;
    assert.equal(p.slot, 999);
  });

  const unsubLeader = bus.onLeader((l) => {
    leaderReceived = true;
    assert.equal(l.currentLeader, 'Validator1');
  });

  bus.emitSlot({
    slot: 999,
    targetSlotDurationMs: 250,
    elapsedMs: 50,
    remainingMs: 200,
    phaseRatio: 0.2,
    driftMs: 0,
    isRealTime: true,
  });

  bus.emitLeader({
    currentLeader: 'Validator1',
    nextLeader: 'Validator2',
    upcomingLeaders: [],
    leaderEstimatedDurationMs: 250,
    handoffState: 'BUILDING',
  });

  assert.equal(slotReceived, true);
  assert.equal(leaderReceived, true);

  unsubSlot();
  unsubLeader();
});
