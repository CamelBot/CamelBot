const Discord = require('discord.js')

module.exports = class commandRunner{
    /**
     * 
     * @param {Discord.CommandInteraction} interaction 
     * @param {Object} externalsource 
     * @param {String} source 
     */
    constructor(interaction=null,externalsource=null,source){
        this.interaction=interaction
        this.externalsource=externalsource
        this.source=source
    }
    /**@type {Discord.CommandInteraction} */
    interaction
    /**@type {Object} */
    externalsource
    /**@type {String} */
    source
}