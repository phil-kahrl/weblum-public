self.onactivate  = (event) => {
  console.log("SERVICE WORKER ACTIVATED");
  //postMessage({foo: "bar"});
};

self.addEventListener('install', (event) => {
  console.log("SERVICE WORKER INSTALLED");
  //postMessage({foo: "bar"});
});

self.addEventListener("fetch", (event) => {
  console.log("fetch called")
  console.log(event);
  //event.respondWith(new Response("Network error happened11111", {
    //status: 408,
    //headers: { "Content-Type": "text/plain" },
  //}));
});