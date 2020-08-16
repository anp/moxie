let timeoutExp = 6;
let changes = null;

function listenToChangeEvents() {
    try {
        changes = new WebSocket(`ws://${(location.host || "[::1]:8000")}/ch-ch-ch-changes`);
    } catch (e) {
        changes.error(e);
    }

    changes.onclose = () => {
        let timeout = Math.pow(2, timeoutExp);
        timeoutExp += 1;
        console.log('livereload socket closed, scheduling reconnect in', timeout, 'ms');
        setTimeout(listenToChangeEvents, timeout);
    };
    changes.onerror = ({ data: error }) => {
        console.error('livereload error', error);
        changes.close();
    };
    changes.onmessage = ({ data }) => {
        location.reload();
    };
}

listenToChangeEvents();