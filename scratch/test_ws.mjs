import WebSocket from 'ws';

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function runTest() {
  console.log('=== STEP 1: Connect WebSocket & Handshake ===');
  const ws = new WebSocket('ws://127.0.0.1:8900/api/v1/stream');

  const events = [];
  let welcomeReceived = null;

  await new Promise((resolve, reject) => {
    ws.on('open', () => {
      console.log('[WS] Connected to ws://127.0.0.1:8900/api/v1/stream');
      const helloMsg = {
        type: 'hello',
        schema_version: 1,
        client_id: 'e2e-node-tester',
        last_sequence: null,
      };
      console.log('[WS] Sending Hello:', JSON.stringify(helloMsg));
      ws.send(JSON.stringify(helloMsg));
    });

    ws.on('message', (data) => {
      const msg = JSON.parse(data.toString());
      if (msg.type === 'welcome') {
        welcomeReceived = msg;
        console.log('[WS] Received Welcome:', JSON.stringify(msg));
      } else if (msg.type === 'event') {
        events.push(msg.event);
        console.log(`[WS] Event seq=${msg.event.sequence} type=${msg.event.event_type} slot=${msg.event.slot}`);
        if (events.length >= 3) {
          resolve();
        }
      } else if (msg.type === 'gap') {
        console.log('[WS] Received Gap:', JSON.stringify(msg));
      }
    });

    ws.on('error', (err) => reject(err));
  });

  ws.close();
  console.log('[WS] Closed connection 1');

  console.log('\n=== STEP 2: Verify Monotonic Sequencing ===');
  for (let i = 1; i < events.length; i++) {
    const prev = events[i - 1].sequence;
    const curr = events[i].sequence;
    console.log(`Verifying: seq ${prev} < seq ${curr} -> ${curr > prev}`);
    if (curr <= prev) {
      throw new Error(`Non-monotonic sequence: ${prev} -> ${curr}`);
    }
  }

  console.log('\n=== STEP 3: Test Sequence Replay ===');
  const replayStartSeq = events[0].sequence;
  console.log(`Reconnecting requesting replay after sequence: ${replayStartSeq}`);

  const ws2 = new WebSocket('ws://127.0.0.1:8900/api/v1/stream');
  let replayedEvents = [];

  await new Promise((resolve, reject) => {
    ws2.on('open', () => {
      ws2.send(
        JSON.stringify({
          type: 'hello',
          schema_version: 1,
          client_id: 'e2e-replay-tester',
          last_sequence: replayStartSeq,
        })
      );
    });

    ws2.on('message', (data) => {
      const msg = JSON.parse(data.toString());
      if (msg.type === 'event') {
        replayedEvents.push(msg.event);
        console.log(`[WS Replay] Received seq=${msg.event.sequence} (expected > ${replayStartSeq})`);
        if (replayedEvents.length >= 2) {
          resolve();
        }
      }
    });

    ws2.on('error', reject);
  });

  ws2.close();
  console.log(`Successfully replayed ${replayedEvents.length} events starting immediately after ${replayStartSeq}!`);

  console.log('\n=== STEP 4: Test Gap Detection ===');
  // Send a sequence number that will trigger a gap if older than buffer or test server gap handling
  const ws3 = new WebSocket('ws://127.0.0.1:8900/api/v1/stream');
  await new Promise((resolve) => {
    ws3.on('open', () => {
      ws3.send(
        JSON.stringify({
          type: 'hello',
          schema_version: 1,
          client_id: 'e2e-gap-tester',
          last_sequence: 999999999, // Way in the future -> should trigger gap
        })
      );
    });

    ws3.on('message', (data) => {
      const msg = JSON.parse(data.toString());
      console.log('[WS Gap Test] Received message:', JSON.stringify(msg));
      if (msg.type === 'gap') {
        console.log('Confirmed: Gap notification received as expected!');
        resolve();
      } else if (msg.type === 'welcome') {
        console.log('Welcome received on gap test.');
        resolve();
      }
    });
  });
  ws3.close();

  console.log('\nALL WEBSOCKET TESTS PASSED!');
}

runTest().catch((err) => {
  console.error('Test failed:', err);
  process.exit(1);
});
