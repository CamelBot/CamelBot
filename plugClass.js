

const winston = require('winston');

const camelLibjs = require('./camelLib');

module.exports = class plugin {
    constructor(mappedClass, commands, manifest){
        this.class=mappedClass;
        this.commands=commands
        this.name=manifest.name
        this.description=manifest.description
        this.manifest=manifest
    }
    /**@type {Class} */
    class;
    /**@type {Array} */
    commands;
    /**@type {String} */
    name;
    /**@type {String} */
    description;
    /**@type {Object} */
    manifest
}
