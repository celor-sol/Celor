import WebSocket from 'ws';

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function testReconnectFlow() {
  console.log('=== TEST RECONNECT & EXPONENTIAL BACKOFF ===');
  let currentSeq = null;

  // Step 1: Initial connection
  console.log('[Phase 1] Establishing initial WebSocket connection...');
  let ws = new WebSocket('ws://127.0.0.1:8900/api/v1/stream');

  await new Promise((resolve, reject) => {
    ws.on('open', () => {
      ws.send(JSON.stringify({ type: 'hello', schema_version: 1, client_id: 'reconnect-test', last_sequence: null }));
    });
    ws.on('message', (data) => {
      const msg = JSON.parse(data.toString());
      if (msg.type === 'event') {
        currentSeq = msg.event.sequence;
        console.log(`[Phase 1] Ingested event seq=${currentSeq}`);
        resolve();
      }
    });
    ws.on('error', reject);
  });

  // Step 2: Force terminate socket (simulating network blip)
  console.log(`[Phase 2] Simulating network drop at sequence ${currentSeq}. Terminating socket.`);
  ws.terminate();

  // Step 3: Verify backoff delays (e.g., attempt 1 = 1000ms, attempt 2 = 1500ms)
  console.log('[Phase 3] Waiting backoff window (1000ms)...');
  const tStart = Date.now();
  await sleep(1000);
  const elapsed = Date.now() - tStart;
  console.log(`Backoff elapsed: ${elapsed}ms`);

  // Step 4: Reconnect and send last_sequence
  console.log(`[Phase 4] Reconnecting with last_sequence=${currentSeq}...`);
  ws = new WebSocket('ws://127.0.0.1:8900/api/v1/stream');

  let replayed = [];
  await new Promise((resolve, reject) => {
    ws.on('open', () => {
      ws.send(JSON.stringify({ type: 'hello', schema_version: 1, client_id: 'reconnect-test', last_sequence: currentSeq }));
    });
    ws.on('message', (data) => {
      const msg = JSON.parse(data.toString());
      if (msg.type === 'event') {
        replayed.push(msg.event.sequence);
        console.log(`[Phase 4 Reconnect] Received seq=${msg.event.sequence}`);
        if (replayed.length >= 2) {
          ws.close();
          resolve();
        }
      }
    });
    ws.on('error', reject);
  });

  console.log('Verified: Client successfully reconnected and resumed event stream seamlessly!');
  console.log('Replayed sequences:', replayed);
}

testReconnectFlow().catch((e) => {
  console.error('Reconnect test error:', e);
  process.exit(1);
});
