<<<<<<< HEAD
const camelLibjs = require("./camelLib")
const winston = require('winston')
=======
>>>>>>> b48662d013d1ad3ec0bf0c83ff855c8c69463047




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