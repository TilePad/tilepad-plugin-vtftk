
const portEl = document.getElementById("port");

tilepad.plugin.getProperties()
    .then((properties) => {
        const port = Number(properties.port ?? 8533);
        portEl.value = port;
    })

portEl.onchange = (event)=> {
    const value = Number(event.target.value);
    tilepad.plugin.send({ type: "PORT_CHANGED", port: value })
}
