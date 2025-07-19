fetch("/ip-logger/log")
    .then(r => {
        console.log("status", r.status);
        return r.text();
    })
    .then(d => console.log("response", d))
    .catch(e => console.error("BIG OOPS", e));
