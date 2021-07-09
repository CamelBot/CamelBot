// Represents a class loaded from the manifest, mostly for intellisense purposes





module.exports = class mappedClass {
    constructor(logger, camellib) {
        this.logger = logger;
        this.camellib = camellib;
    }
    /**@type {import('winston').Logger} */
    logger
    /**@type {import('./camelLib')} */
    camellib
};