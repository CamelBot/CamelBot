

const winston = require('winston');

const camelLibjs = require('./camelLib');

module.exports = class plugin {
    constructor(mappedClass, commands, name, description){
        this.class=mappedClass;
        this.commands=commands
        this.name=name
        this.description=description
    }
    /**@type {Class} */
    class;
    /**@type {Array} */
    commands;
    /**@type {String} */
    name;
    /**@type {String} */
    description;
}
