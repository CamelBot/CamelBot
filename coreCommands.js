const { EventEmitter } = require('events');
const Discord = require('discord.js');

/**@type {camelLibjs} */
let camellib;

module.exports = class plugin extends EventEmitter {
    constructor(parameters) {
        super();
        camellib = parameters.camellib;
    }

    /**@type {Discord.Client} */
    client

    initClient(client) {
        this.client = client;
        this.client.on('interaction', interaction => {
            if (interaction.isButton()) {
                /**@type {Object} The parsed JSON from the button interaction ID */
                let buttonInteraction;
                try {
                    // This plugin uses the button ID in a stringified JSON for passing values
                    buttonInteraction = JSON.parse(interaction.customID);
                } catch (err) {
                    console.log(err);
                    return;
                }
                // If it happens to parse as a JSON, check if it follows the property
                if (!Object.prototype.hasOwnProperty.call(buttonInteraction, 'command')) return;
                // Check if the command is this one
                if (buttonInteraction.command == 'plugins') {
                    // Only an admin can change plugin toggles
                    if (!interaction.member.permissions.has('ADMINISTRATOR')) {
                        let toSend = new Discord.MessageEmbed()
                            .setTitle('Error')
                            .setColor('#ff0000')
                            .setTimestamp()
                            .addField('Permission', 'You do not have permission to edit the server\'s plugins');
                        interaction.reply({ embeds: [toSend], ephemeral: true });
                        return;
                    }
                    if (!Object.prototype.hasOwnProperty.call(buttonInteraction, 'button')) return;
                    if (buttonInteraction.button == 'disable') {
                        // Get a list of enabled plugins 
                        let enabledPlugins = camellib.database.get(interaction.guild.id).enabledPlugins;
                        // Chop it from the list
                        enabledPlugins.splice(enabledPlugins.indexOf(buttonInteraction.plugin));
                        // Save the database
                        camellib.saveDatabase();
                        // Purge all the unecessary commands
                        camellib.purgeCommands();
                        // Let all plugins know that a plugin has been disabled
                        camellib.emit('pluginDisabled', interaction.guild.id, buttonInteraction.plugin);
                        let toSend = new Discord.MessageEmbed()
                            .setColor('#340034')
                            .setTitle(buttonInteraction.plugin + ' disabled')
                            .addField('Success', 'All commands and features are now disabled for your Discord Server')
                            .setTimestamp();
                        interaction.reply({ embeds: [toSend] });
                        return;
                    }
                    if (buttonInteraction.button == 'enable') {
                        let enabledPlugins = camellib.database.get(interaction.guild.id).enabledPlugins;
                        // Make sure we aren't writing a duplicate
                        if (!enabledPlugins.includes(buttonInteraction.plugin)) {
                            enabledPlugins.push(buttonInteraction.plugin);
                            camellib.saveDatabase();
                            camellib.publishCommands();
                        }
                        // Even if it exists, we can make them believe that they enabled it ;)
                        camellib.emit('pluginEnabled', interaction.guild.id, buttonInteraction.plugin);
                        let toSend = new Discord.MessageEmbed()
                            .setColor('#008000')
                            .setTitle(buttonInteraction.plugin + ' enabled')
                            .addField('Success', 'All commands and features are now active in your server')
                            .setTimestamp();
                        interaction.reply({ embeds: [toSend] });
                        return;
                    }
                }


            }
        });
    }

    /**
     * 
     * @param {commandRunner} commandRunner TODO: fix this
     */
    help(commandRunner) {
        if (commandRunner.source == 'discord') {
            commandRunner.interaction.reply('ha ha no');
        }

    }
    /**
     * 
     * @param {commandRunner} commandRunner 
     */
    pluginCommand(commandRunner) {
        if (commandRunner.source == 'discord') {
            const toSend = new Discord.MessageEmbed()
                .setTitle('**__Plugins__**')
                .addField('Installed plugins', 'Below are all commands installed in CamelBot, enable or disable them here.')
                .setColor(generateColor());
            commandRunner.interaction.reply({ embeds: [toSend] });
            camellib.plugins.forEach(plugin => {
                /**@type {plugClass} */
                let thisplug = plugin;

                let embed = new Discord.MessageEmbed()
                    .setTitle('__' + thisplug.name + '__')
                    .addField('Description', thisplug.description)
                    .setColor(generateColor());

                let comp = new Discord.MessageActionRow();
                if (camellib.database.get(commandRunner.interaction.guild.id).enabledPlugins.includes(thisplug.name)) {
                    comp.addComponents(
                        new Discord.MessageButton()
                            .setCustomID(JSON.stringify({
                                'command': 'plugins',
                                'plugin': thisplug.name,
                                'button': 'disable'
                            }))
                            .setLabel('disable')
                            .setStyle('DANGER')
                    );
                } else {
                    comp.addComponents(
                        new Discord.MessageButton()
                            .setCustomID(JSON.stringify({
                                'command': 'plugins',
                                'plugin': thisplug.name,
                                'button': 'enable'
                            }))
                            .setLabel('enable')
                            .setStyle('PRIMARY')
                    );
                }


                commandRunner.interaction.followUp({ embeds: [embed], components: [comp] });
            });


        }

    }
};



/**
 * @returns {String} Returns a random color in hex
 */
function generateColor() {
    const allowedHex = ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F'];
    let toSend = '#';
    for (let i = 0; i < 6; i++) {
        toSend += allowedHex[Math.floor(Math.random() * allowedHex.length)];
    }
    return (toSend);
}