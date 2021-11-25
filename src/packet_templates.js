// jkcoxson
// Packet templates, serving as a lazy API doc
console.log("This file is not meant to be run");

_ = [
    // All packets are formated as JSON objects
    // Depending on the connection type, they will be piped through stdout or a TCP socket following '\n'
    // All packets must include a type, this will be used to determine how to parse the packet


    // Event packets
    // These packets are sent when an event occurs that needs to be broadcasted
    // Only components that have requested the event type will receive the packet
    // For example, if I have a spam detection plugin, I will request the 'message' event
    {
        type: "event",
        event: "message",
        author: "jkcoxson",
        message: "Hello, world!",
        // Anything can be put in an event packet, as long as the receiving component is aware of extra data
    },

    // Send packets
    // These packets are sent when a component wants to send data to another component
    {
        type: "send",
        target: "my_fancy_component",
        banana: "yellow",
        apple: "red",
    },

    // Intents packets
    // These packets are to let the core know what the component is trying to do and what data to send
    // All these fields are required, but are only read in certain circumstances
    {
        type: "intents",
        events: ["message", "explosion", "yeet", "channel_create"], // These only matter for plugins
        commands: [{
            name: "fancycommand",
            description: "A fancy command",
            options: [
                {
                    name: "suit",
                    description: "Wear a suit",
                    type: "bool",
                    required: false
                }
            ]
        }]
    },

    // Command packets
    // These packets are sent from an interface to the core to execute a command, and then to the plugin that created it
    {
        type: "command",
        name: "fancycommand",
        source: "fancy interface", // The name of the interface that sent the command
        // Any fields after this are up to the interface to define
        data: "anything can go here, the plugin must know what to expect"
    },

    // Sniffer packets
    // These packets are sent if the component is a sniffer
    // This packet must be returned to the core for it to be passed on to the destination component
    // If it is not returned, the packet will be dropped
    {
        type: "sniffer",
        source: "interface_1",
        destination: "interface_2",
        event: "",
        sniffers: [], // No touchy, the core manages this. It is kept here for multi-threaded purposes.
        packet: {} // The entire packet 
        // The packet may be mutilated or dropped, depending on the purpose of the sniffer
    }

]