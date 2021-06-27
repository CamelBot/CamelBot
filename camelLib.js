// General Library for all plugins to tap into. Yay, classes
// jkcoxson


const { EventEmitter } = require('events');
const { resolve } = require('path');
const fs = require('fs')
const coreLibjs = require('./coreCommands');
const { Client } = require('discord.js');
const winston = require('winston');

module.exports = class camellib extends EventEmitter {
    constructor(parameters){
        super();
        this.private=parameters.private;
        this.database = new Map();
        this.plugins = new Map();
        this.mappedClasses = new Map();
        this.mappedCommands = new Map();
        parameters.database.forEach(guild=>{
            this.database.set(guild.id,guild)
        })
        this.saveDatabase()
    }
    /**@type {Object} */
    private;
    /**@type {Client} */
    client;
    /**@type {Map} */
    database;
    /**@type {winston.Logger} */
    logger;
    plugins;
    mappedClasses;
    mappedCommands;
    coreLib = new coreLibjs({
        "logger":this.logger,
        "camellib":this
    });
    coreCommands = [
        {
            "name":"help",
            "description":"Get help using CamelBot",
            "class":"coreCommands.js",
            "method":"help",
            "source":[
                "discord",
                "minecraft"
            ],
            "options":[]
        },
        {
            "name":"plugins",
            "description":"Turn on or off a command",
            "class":"coreCommands.js",
            "method":"pluginCommand",
            "source":[
                "discord",
                "minecraft"
            ],
            "options":[]
        }
    ]
    mapCoreCommands(){
        this.mappedClasses.set("core/coreCommands.js",this.coreLib)
        this.coreCommands.forEach(command=>{
            this.mappedCommands.set(command.name, {
                "manifest":command,
                "method":this.coreLib[command.method],
                "plugin":"core"
            })
        })
        this.coreLib.initClient(this.client)
    }

    /**
     * Initiates commands if they aren't already created. Once
     */
    publishCommands(){
        this.database.forEach(guild=>{
            this.mappedCommands.forEach(command=>{
                if((command.plugin=="core"||guild.enabledPlugins.includes(command.plugin))&&command.manifest.source.includes('discord')){
                    this.client.guilds.cache.get(guild.id).commands.fetch().then(()=>{
                        if(!this.client.guilds.cache.get(guild.id).commands.cache.find(element => element.name==command.manifest.name)){
                            this.client.guilds.cache.get(guild.id).commands.create({
                                name: command.manifest.name,
                                description: command.manifest.description,
                                options: command.manifest.options
                            })
                            this.emit("commandCreated",(guild.id,command.manifest.name))
                        }
                    })
                }
            })
        })
        
    }

    /**
     * Gets rid of the commands that are in the Discord server but shouldn't be
     */
    purgeCommands(){
        this.database.forEach(guild=>{
            this.client.guilds.cache.get(guild.id).commands.fetch().then(()=>{
                this.client.guilds.cache.get(guild.id).commands.cache.forEach(command=>{
                    let found = false;
                    this.mappedCommands.forEach(element=>{
                        if(this.database.get(guild.id).enabledPlugins.includes(element.plugin)&&element.manifest.name==command.name){
                            found = true;
                        }
                        if(element.plugin=="core"&&element.manifest.name==command.name){
                            found = true;
                        }
                    })
                    if(!found){
                        command.delete();
                        this.emit("commandDeleted",(guild.id,command.name))
                    }
                })
            })
            
        })
    }

    async saveDatabase(){
        let toSend = []
        this.database.forEach(guild=>{
            toSend.push(guild)
        })
        fs.writeFileSync('./configs/database.json',JSON.stringify(toSend, null, "\t"))
    }

}
