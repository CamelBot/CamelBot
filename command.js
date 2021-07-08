module.exports = class command {
    constructor(manifest, method, plugin) {
        this.manifest = manifest;
        this.method = method;
        this.plugin = plugin;
    }
    /**@type {Object} The JSON object of the manifest*/
    manifest;
    /**@type {Function} The method the command will run*/
    method;
    /**@type {String} The name of the plugin the command belongs to*/
    plugin;
};