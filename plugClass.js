

const winston = require('winston');

const camelLibjs = require('./camelLib');

module.exports = class plugin {
<<<<<<< HEAD
    constructor(mappedClass, commands, manifest){
        this.class=mappedClass;
        this.commands=commands
        this.name=manifest.name
        this.description=manifest.description
        this.manifest=manifest
=======
    constructor(mappedClass, commands, name, description){
        this.class=mappedClass;
        this.commands=commands
        this.name=name
        this.description=description
>>>>>>> b48662d013d1ad3ec0bf0c83ff855c8c69463047
    }
    /**@type {Class} */
    class;
    /**@type {Array} */
    commands;
    /**@type {String} */
    name;
    /**@type {String} */
    description;
<<<<<<< HEAD
    /**@type {Object} */
    manifest
=======
>>>>>>> b48662d013d1ad3ec0bf0c83ff855c8c69463047
}
