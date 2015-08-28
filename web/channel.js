import {EventEmitter} from 'events';


let channels = Object.create(null);

export default class Channel extends EventEmitter {
    constructor(name) {
        if (name in channels)
            return channels[name];

        super();
        channels[name] = this;

        let socket = this.socket = new WebSocket(`ws://${location.host}/${name}`);
        socket.binaryType = 'arraybuffer';

        socket.onopen = () => this.emit('connect');
        socket.onmessage = e => this.emit('data', e.data);

        socket.onclose = e => {
            if (!e.wasClean)
                this.emit('error', e);

            this.emit('disconnect');
            delete channels[name];
        };
    }

    disconnect() {
        this.socket.close();
    }
}
