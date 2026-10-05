import WebSocket from 'ws';

async function client(name) {
  const ws = new WebSocket('ws://127.0.0.1:8900/api/v1/stream');
  const seqs = [];
  return new Promise((resolve, reject) => {
    ws.on('open', () => {
      ws.send(JSON.stringify({ type: 'hello', schema_version: 1, client_id: name, last_sequence: null }));
    });
    ws.on('message', (data) => {
      const msg = JSON.parse(data.toString());
      if (msg.type === 'event') {
        seqs.push(msg.event.sequence);
        console.log(`[${name}] Received seq=${msg.event.sequence}`);
        if (seqs.length >= 3) {
          ws.close();
          resolve(seqs);
        }
      }
    });
    ws.on('error', reject);
  });
}

async function run() {
  console.log('Connecting 3 concurrent clients to Chrono Service...');
  const results = await Promise.all([client('client-A'), client('client-B'), client('client-C')]);
  console.log('All 3 clients received events:', results);
  console.log('Multi-client test: SUCCESS');
}

run().catch((e) => {
  console.error('Multi-client failed:', e);
  process.exit(1);
});
