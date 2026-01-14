// The actor interface
const Gallery = {
    id: undefined,
    shared_files: [],
    message_handler: _ => {},
    reconnect_handler: _ => {},
    connected: false,
};

window.addEventListener('load', function() {
    'use strict';
    let galleries = [];
    let selected_gallery_id = undefined;
    const image_cache /*: Map<id, Map<String, CachedPicture>> */ = {};
    const galleries_element = document.querySelector('#galleries');
    const galleries_list = galleries_element.querySelector('ul');
    const gallery_view = galleries_element.querySelector('div');

    Gallery.message_handler = message_handler;
    Gallery.reconnect_handler = reconnect_handler;
    Gallery.update_ui = update_ui;

    update_ui();
    update_view();


function reconnect_handler(online) {
    if (online) {
        ws.send_object({"ClientRequest":"ListAllPods"});
    } else {
        clear_image_cache();

        // drop other data
        galleries = [];
    }
}

function message_handler(message) {
    if (typeof message.Pods !== 'undefined') {
        // Sometimes NewPod messages arrive before Pods message on reconnect
        galleries = galleries.concat(message.Pods);
        clear_image_cache();
        galleries.forEach(p => image_cache[p.id] = []);
        update_ui();
    } else
    if (typeof message.NewPod !== 'undefined') {
        message.NewPod.paths = message.NewPod.paths || [];
        galleries.push(message.NewPod);
        image_cache[message.NewPod.id] = [];
        update_ui();
    } else
    if (typeof message.UnknownPod !== 'undefined') {
        Gallery.reconnect_handler();
    } else
    if (typeof message.PodGone !== 'undefined') {
        galleries = galleries.filter(x => x.id !== message.PodGone);
        update_ui();
        if (selected_gallery_id === message.PodGone) {
            update_view();
        }
    } else
    if (typeof message.PodUpdateName !== 'undefined') {
        galleries[indexOfPod(message.PodUpdateName.id)].name = message.PodUpdateName.name;
        update_ui();
    } else
    if (typeof message.PodUpdatePaths !== 'undefined') {
        const id = message.PodUpdatePaths.id;
        const pod_index = indexOfPod(id);
        const replace_images = message.PodUpdatePaths.replace_images;
        const last_modified = new Date(message.PodUpdatePaths.last_modified)

        galleries[pod_index].paths = message.PodUpdatePaths.paths;
        galleries[pod_index].last_modified = last_modified;

        // prepare for Cache Layer
        if (replace_images || typeof image_cache[id] !== 'object') {
            image_cache[id] = [];
        }

        update_ui();
        if (id === selected_gallery_id) {
            update_view();
        }
    } else
    if (typeof message.DeliverImage !== 'undefined') {
        const id = message.DeliverImage.gallery_id;
        image_cache[id][message.DeliverImage.path].save_blob(message.DeliverImage.blob);
        if (id === selected_gallery_id) {
            update_view();
        }
    } else {
        error(['client_response unimplemented', message]);
    }
}

function indexOfPod(id) {
    for (let i = 0; i < galleries.length; ++i) {
        if (galleries[i].id === id) {
            return i;
        }
    }
    return undefined;
}

/// Update the list of galleries
function update_ui() {
    if (galleries.length === 0) {
        galleries_list.innerHTML = "<h3>No Gallery connected</h3>";
    } else {
        galleries.sort((a, b) => a.name.localeCompare(b.name));
        galleries_list.innerHTML = '';

        galleries.forEach(x => {
            const div = document.createElement('li');
            const title = document.createElement('h3');
            const text = document.createElement('div');

            title.innerText = x.name || `unnamed Gallery #${x.id}`;
            text.innerHTML = x.paths.length === 0 ? 'No images' : `${x.paths.length} images <br> last last_modified: ${x.last_modified}`;

            div.appendChild(title);
            div.appendChild(text);

            div.addEventListener('click', ev => {
                // switch to that gallery
                selected_gallery_id = x.id;
                update_view();
            });

            galleries_list.appendChild(div);
        });

    }
}

/// Update the Gallery
function update_view() {
    if (selected_gallery_id === undefined) {
        gallery_view.innerHTML = '<div class="centered">No Gallery selected</div>';
        return;
    }

    const i = indexOfPod(selected_gallery_id);
    if (i === undefined) {
        gallery_view.innerHTML = `<div class="centered">Invalid Gallery Id: ${selected_gallery_id}</div>`;
        return;
    }

    gallery_view.innerHTML = '';
    galleries[i].paths.forEach(path => {
        let cp = image_cache[selected_gallery_id][path];
        if (cp === undefined) {
            cp = new CachedPicture(selected_gallery_id, path);
            image_cache[selected_gallery_id][path] = cp;
        } else {
            cp.reappend_to_gallery();
        }
    });

}

function CachedPicture(id, path) {
    this.gallery_id = id;
    this.path = path;

    this.div = document.createElement('div');

    this.img = document.createElement('img');
    this.div.appendChild(this.img);


    const text = document.createElement('div');
    text.innerText = path;
    this.div.appendChild(text);

    this.reappend_to_gallery();
}

CachedPicture.prototype = {
    reappend_to_gallery: function() {
        this.cache_update();
        gallery_view.appendChild(this.div);
    },
    cache_update: function() {
        if (this.blob === undefined) {
            ws.send_object({"ClientRequestAsync": {
                "RequestImage": {
                    "gallery_id": this.gallery_id,
                    "path": this.path,
                },
            }});
        } else {
            this.img.src = this.blob;
        }
    },
    save_blob: function(blob) {
        this.blob = blob;
    },
};

function clear_image_cache() {
    for (var k in image_cache) {
        delete image_cache[k];
    }
}

});
