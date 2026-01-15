
const log = console.log
    , error = console.error;

let html_logger = _ => {};
let ws = undefined;
setup_ws();

WebSocket.prototype.send_object = function(obj) {
    const str = JSON.stringify(obj);
    html_logger('> ' + str.substr(0, 256));
    return this.send(str);
}

window.addEventListener('load', function() {
    html_logger = make_logger(
        document.querySelector('#ws_echo_out'),
        document.querySelector('#ws_state')
    );

    setup_test_websocket();
});


function setup_test_websocket() {
    const button = document.querySelector('#ws_echo');


    button.addEventListener('click', _ => {
        const entry = `> readyState: ${ws.readyState} - OPEN: ${ws.readyState === ws.OPEN}`;

        log(entry);
        html_logger(entry);

        //ws.send_object({"ClientRequest":"ListAllPods"});
        ws.send_object({"ClientRequest":{"ListPodStructure":2}});
    });
}

function make_logger(element, state_element) {
    const now = Date.now;
    return function(str) {
        element.innerHTML += `${now()}: ${str}\n`;
        state_element.innerText = `Pod { connected: ${Pod.connected}, registered: ${Pod.registered}, } `;
    }
}

function setup_ws() {
    // automatically enable WebSocket over TLS
    //ws = new WebSocket('ws'+(location.protocol.indexOf('https') === 0 ? 's' : '')+'://'+location.host+'/ws/');
    ws = new WebSocket('ws'+'://'+location.host+'/ws');

    ws.onclose = _ => {
        html_logger(`-.-.-.-.-.-.-.-.-.-.-.-.-.-.-.- LOST WebSocket Connection -.-.-.-.-.-.-.-.-.-.-.-.-.-.-.-`);
        Gallery.reconnect_handler(false);
        Pod.reconnect_handler(false);

        // TODO add more than one retry here
        requestAnimationFrame(_ => {
            log('reconnecting WebSocket in a second');
            setTimeout(setup_ws, 1024);
        });
    }

    ws.onmessage = msg => {
        log(msg.data.substr(0, 42));
        html_logger('< ' + msg.data.substr(0, 256));
        const data = JSON.parse(msg.data);
        if (typeof data.ClientResponse !== 'undefined') {
            Gallery.message_handler(data.ClientResponse);
        } else
        if (typeof data.PodResponse !== 'undefined') {
            Pod.message_handler(data.PodResponse);
        } else {
            console.error(["unimplemented message", data]);
            html_logger('! ' + msg.data);
        }
    };

    ws.onopen = _ => {
        log('Connected.');
        html_logger(`-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+- ESTABLISHED WebSocket Connection -+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-`);

        Gallery.reconnect_handler(true);
        Pod.reconnect_handler(true);
    }
}
