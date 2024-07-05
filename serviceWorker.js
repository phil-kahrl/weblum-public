self.onactivate  = (event) => {
  console.log("SERVICE WORKER ACTIVATED");
  console.log(event);
  //postMessage({foo: "bar"});
};

self.addEventListener('install', (event) => {
  console.log("SERVICE WORKER INSTALLED");
  //postMessage({foo: "bar"});
});

self.addEventListener("fetch", (event) => {
  console.log("fetch called");
  console.log(event);
  const channel = new BroadcastChannel("test")
  if (event.request.method === "POST") {
    console.log("processing POST request");
    event.request.arrayBuffer().then( (buffer) => {
      let binary = '';
      const bytes = new Uint8Array(buffer);
      const len = bytes.byteLength;
      for (let i = 0; i < len; i++) {
        binary += String.fromCharCode(bytes[i]);
      }
      const base64 = btoa(binary);
      channel.postMessage({payload: base64});
      console.log("message sent");
    }).catch( (err) => {
      console.log(err);
      console.log("error gettng the blob");
    });
  }
});