self.onactivate  = (event) => {
  console.log("SERVICE WORKER ACTIVATED");
};

self.addEventListener('install', (event) => {
  console.log("SERVICE WORKER INSTALLED");
});

const channel = new BroadcastChannel("file_share");
const asyncResponse = async (event) => {
  try {
    const formData = await event.request.formData();
    const keys = formData.keys();
    //console.log(keys);
    for (k of keys) {
      console.log(k);
      const val = formData.get(k);
      console.log(val);
      const buffer = await val.arrayBuffer();
      let binary = '';
      const bytes = new Uint8Array(buffer);
      const len = bytes.byteLength;
      for (let i = 0; i < len; i++) {
        binary += String.fromCharCode(bytes[i]);
      }
      const base64 = btoa(binary);
      channel.postMessage({payload: base64});
    }
    //const buffer = await event.request.arrayBuffer();
   //const data = await event.request.formData();
    //let binary = '';
    //const bytes = new Uint8Array(buffer);
    //const len = bytes.byteLength;
    //for (let i = 0; i < len; i++) {
      //binary += String.fromCharCode(bytes[i]);
    //}
    //const base64 = btoa(binary);
    //channel.postMessage({payload: base64});
    return new Response('success')
  } catch(err) {
    console.log(err);
    return new Response('error')
  }
}

self.addEventListener("fetch", async (event) => {
  console.log("fetch")
  if (event.request.method === "POST") {
   await event.respondWith(asyncResponse(event));
  }
});