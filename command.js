




module.exports = class command {
    constructor(manifest,method,plugin){
        this.manifest=manifest
        this.method=method
        this.plugin=plugin
    }
    /**@type {Object} */
    manifest;
    /**@type {Function} */
    method;
    /**@type {String} */
    plugin;
}