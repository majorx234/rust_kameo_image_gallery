window.onerror = function(error, url, line) {
    const message = `${url}#${line}: ${error}`;
    const output = document.querySelector('#message');

    if (!output) {
        alert(message);
    } else {
        output.innerText = message;
    }
}
