self.onactivate  = (event) => {
  console.log("SERVICE WORKER ACTIVATED");
};

self.addEventListener('install', (event) => {
  console.log("SERVICE WORKER INSTALLED");
});

self.addEventListener("fetch", (event) => {
  const channel = new BroadcastChannel("file_share")
  if (event.request.method === "POST") {
    event.request.arrayBuffer().then( (buffer) => {
      let binary = '';
      const bytes = new Uint8Array(buffer);
      const len = bytes.byteLength;
      for (let i = 0; i < len; i++) {
        binary += String.fromCharCode(bytes[i]);
      }
      const base64 = btoa(binary);
      channel.postMessage({payload: base64});
    }).catch( (err) => {
      console.log(`error gettng the blob ${err}`);
    });
  }
});