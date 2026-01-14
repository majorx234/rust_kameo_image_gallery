// The actor interface
const Pod = {
    id: Math.round(Math.random() * 100000),
    shared_files: [],
    message_handler: _ => {},
    reconnect_handler: _ => {},
    connected: false,
    registered: false,
};

window.addEventListener('load', function() {
    'use strict';

    const inputElement = document.querySelector('#pod_share');
    inputElement.addEventListener("change", handleFiles, false);

    const pod_name = document.querySelector('#pod_name');
    pod_name.addEventListener("change", updatePreviewTitle, false);
    pod_name.addEventListener("keyup", updatePreviewTitle, false);
    // catch form submit
    inputElement.parentElement.addEventListener("submit", ev => {
        registerSelf();
    }, false);

    Pod.message_handler = message_handler;
    Pod.reconnect_handler = reconnect_handler;

    inputElement.value = '';
    regenPreview();
    updatePreviewTitle();

function message_handler(message) {
    console.log(['Pod::message_handler()', message]);
    if (typeof message.Registered !== 'undefined') {
        Pod.id = message.Registered.global_id;
        updatePreviewTitle();
        Pod.registered = true;
        if (Pod.shared_files.length > 0) {
            publishPictures(true);
        }
    } else
    if (typeof message.AlreadyRegistered !== 'undefined') {
        Pod.id = message.AlreadyRegistered.global_id;
        updatePreviewTitle();
        Pod.registered = true;
        html_logger("you are already sharing");
    } else
    if (typeof message.RequestImage !== 'undefined') {
        const client_id = message.RequestImage.client_id;
        const path = message.RequestImage.path;

        const candidate = Pod.shared_files.find(file => file.name === path);
        if (candidate === undefined) {
            // TODO send an error: ws.send_object()
            console.error(["no candidate found for path", path])
        } else {
            if (!candidate.blob) {
                console.error(['candidate missing blob', candidate]);
                debugger;
            }
            ws.send_object({"PodRequest":{
                "DeliverImage":{
                    "client_id": client_id,
                    "path": path,
                    "blob": candidate.blob,
                }
            }});
        }
    }
    else {
        error(['pod_response unimplemented', message]);
    }
}
function reconnect_handler(connected) {
    console.log(['Pod::reconnect_handler()', connected]);
    Pod.connected = connected;

    if (connected && Pod.shared_files.length > 0) {
        registerSelf();
    }

    if (connected === false) {
        Pod.registered = false;
    }
}


function handleFiles() {
    const files = this.files; /* now you can work with the file list */
    log(["handleFiles", files]);
    if (typeof files === 'undefined') {
        return;
    }

    let replace_images = false;
    for (let i = 0, numFiles = files.length; i < numFiles; ++i) {
        const file = files[i];
        if (file.type.indexOf('image/') === 0) {
            let old_len = Pod.shared_files.length;
            Pod.shared_files = Pod.shared_files.filter(f => f.name !== file.name);
            if (Pod.shared_files.length < old_len) {
                replace_images = true;
            }
            Pod.shared_files.push(file);
        } else {
            console.error(['Pod::handleFiles() invalid file type', file]);
        }
    }

    publishPictures(replace_images);
    regenPreview();
}

function registerSelf() {
    ws.send_object({
        "PodRequest":{
            "RegisterSelf":{ "proposed_id": typeof Pod.id === 'number' ? Pod.id : null, "name": normalized_title() }
        }
    });
}

function publishPictures(replace_images) {
    if (Pod.connected) {
        if (Pod.registered === false) {
            registerSelf();
        }
        const paths = Pod.shared_files.reduce((b, cur) => { b.push(cur.name); return b }, []);
        ws.send_object({"PodRequest": {"UpdatePaths": { "paths": paths, "replace_images": replace_images, } } });
    }
}

function updatePreviewTitle() {
    const title = document.querySelector('#pod_preview h1');
    const name = normalized_title();
    title.innerText = name;

    if (Pod.registered) {
        // TODO delay until the user stops typing?
        ws.send_object({"PodRequest":{"UpdateTitle":{"name":name}}});
    }
}
function normalized_title() {
    const pod_name = document.querySelector('#pod_name');

    return pod_name.value.trim() || 'Unnamed Gallery #'+Pod.id;
}

function regenPreview() {
    const preview = document.querySelector('#pod_preview div');
    const event_to_src = (function(aImg, file_handler) {
        const MAX_BLOB_LENGTH = 64*1024;
        function try_thumb(target_size, original, file_handler) {
            const img = new Image();
            img.onload = function() {
                const width = Math.min(target_size, img.naturalWidth);
                const scaleFactor = width / img.naturalWidth;
                const height = scaleFactor * img.naturalHeight;
                console.log([width, height]);
                const canvas = document.createElement("canvas")
                    , ctx = canvas.getContext("2d");

                canvas.width = width;
                canvas.height= height;

                // draw the img into canvas
                ctx.drawImage(this, 0, 0, width, height);
                const blob = canvas.toDataURL("image/jpeg");

                if (blob.length > MAX_BLOB_LENGTH) {
                    target_size /= 2;
                    try_thumb(target_size, original, file_handler);
                } else {
                    aImg.src = blob;
                    // backup into RAM
                    file_handler.blob = blob;
                }
            };
            img.src = original;
        }

        return function(e) {
            const original = e.target.result;
            if (original.length > MAX_BLOB_LENGTH) {
                const target_size = 512;

                try_thumb(target_size, original, file_handler);
            } else {
                aImg.src = original;
                // backup into RAM
                file_handler.blob = original;
            }
        };
    });

    if (Pod.shared_files.length === 0) {
        preview.innerHTML = '<div>No files shared, add some to see the previews</div>';
    } else {
        preview.innerHTML = '';
    }

    Pod.shared_files.forEach(file => {
        const div = document.createElement("div");
        const text = document.createElement("div");
        const img = document.createElement("img");
        img.classList.add("obj");

        const reader = new FileReader();
        reader.onload = event_to_src(img, file);
        reader.readAsDataURL(file);

        text.innerText = file.name;

        div.appendChild(img);
        div.appendChild(text);
        preview.appendChild(div);
    })
}


});
