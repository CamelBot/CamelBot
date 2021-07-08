// General Library for all plugins to tap into. Yay, classes
// jkcoxson

const {
    EventEmitter
} = require('events');
const fs = require('fs');
const coreLibjs = require('./coreCommands');

module.exports = class camellib extends EventEmitter {
    constructor(parameters) {
        super();
        this.private = parameters.private;
        this.database = new Map();
        this.plugins = new Map();
        this.mappedClasses = new Map();
        this.mappedCommands = new Map();
        parameters.database.forEach(guild => {
            this.database.set(guild.id, guild);
        });
        this.saveDatabase();
    }
    /**@type {Object} Stores the private JSON that contains environment specific data*/
    private;
    /**@type {discord.Client} The discord.js client */
    client;
    /**@type {Map} A map of database records with keys as a guild id */
    database;
    /**@type {winston.Logger} Core logger */
    logger;
    /**@type {Map<plugClass>} Map of plugins, keyed by plugin name */
    plugins;
    /**@type {Map<mappedClass>} Map of classes defined in each manifest, keyed by pluginName/className */
    mappedClasses;
    /**@type {Map<command>} Map of keyed command names to methods and manifests */
    mappedCommands;
    /**@type {coreLibjs} A class for the core commands of CamelBot*/
    coreLib = new coreLibjs({
        'logger': this.logger,
        'camellib': this
    });
    /**@type {Array<Object>} Basically the manifest for the core commands so they can be loaded by the normal loader */
    coreCommands = [{
        'name': 'help',
        'description': 'Get help using CamelBot',
        'class': 'coreCommands.js',
        'method': 'help',
        'source': [
            'discord',
            'minecraft'
        ],
        'options': []
    },
    {
        'name': 'plugins',
        'description': 'Turn on or off a command',
        'class': 'coreCommands.js',
        'method': 'pluginCommand',
        'source': [
            'discord',
            'minecraft'
        ],
        'options': []
    }
    ]
    /**
     * Adds them to the map since they weren't loaded by the manifest loader
     */
    mapCoreCommands() {
        this.mappedClasses.set('core/coreCommands.js', this.coreLib);
        this.coreCommands.forEach(command => {
            this.mappedCommands.set(command.name, {
                'manifest': command,
                'method': this.coreLib[command.method],
                'plugin': 'core'
            });
        });
        this.coreLib.initClient(this.client);
    }

    /**
     * Initiates commands if they aren't already created. IF THEY ALREADY EXIST THEY WILL NOT BE OVERWRITTEN
     */
    publishCommands() {
        this.database.forEach(guild => {
            this.mappedCommands.forEach(command => {
                if ((command.plugin == 'core' || guild.enabledPlugins.includes(command.plugin)) && command.manifest.source.includes('discord')) {
                    this.client.guilds.cache.get(guild.id).commands.fetch().then(() => {
                        if (!this.client.guilds.cache.get(guild.id).commands.cache.find(element => element.name == command.manifest.name)) {
                            this.client.guilds.cache.get(guild.id).commands.create({
                                name: command.manifest.name,
                                description: command.manifest.description,
                                options: command.manifest.options
                            });
                            // If plugins want to edit functions, they can know that it exists now
                            this.emit('commandCreated', (guild.id, command.manifest.name));
                        }
                    });
                }
            });
        });

    }

    /**
     * Gets rid of the commands that are in the Discord server but shouldn't be
     */
    purgeCommands() {
        this.database.forEach(guild => {
            this.client.guilds.cache.get(guild.id).commands.fetch().then(() => {
                this.client.guilds.cache.get(guild.id).commands.cache.forEach(command => {
                    let found = false;
                    this.mappedCommands.forEach(element => {
                        if (this.database.get(guild.id).enabledPlugins.includes(element.plugin) && element.manifest.name == command.name) {
                            found = true;
                        }
                        if (element.plugin == 'core' && element.manifest.name == command.name) {
                            found = true;
                        }
                    });
                    if (!found) {
                        command.delete();
                        this.emit('commandDeleted', (guild.id, command.name));
                    }
                });
            });

        });
    }
    /**
     * Converts the database map back to a JSON and saves it
     */
    async saveDatabase() {
        let toSend = [];
        this.database.forEach(guild => {
            toSend.push(guild);
        });
        fs.writeFileSync('./configs/database.json', JSON.stringify(toSend, null, '\t'));
    }

};