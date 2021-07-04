const camelLibjs = require("./camelLib")
const winston = require('winston')



module.exports = class mappedClass {
    constructor(logger,camellib){
        this.logger=logger
        this.camellib=camellib
    }
    /**@type {winston.Logger} */
    logger
    /**@type {camelLibjs} */
    camellib
}