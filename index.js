// CamelBot 4, but this time he's mad
// Let's try not to hate everything edition
// jkcoxson

console.log("System Startup, all hail camels")
const splashScreen = "                                                          `7wy$AOBBB@@@Df^`                         \n" +
"                                                        ?BB@@@@@@@@@@@@@@@ga7:                      \n" +
"              ```:,                                     X@@@@@@@@@@@@@@@@@@@@@X~                    \n" +
"          `|hQ@@@@@:                                   x@@@@@@@@@@@@@@@@@@@@@@@@p*.                 \n" +
"    ';cI2O@@@@@@@@@8:                          `+*<?IqB@@@@@@@@@@@@@@@@@@@@@@@@@@@@QA1r:'           \n" +
" ,mB@@@@@@@@@@@@@@@@X                       .tpB@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@B$7,       \n" +
"|@@@@@@@@@@@@@@@@@@@@'                    `|B@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@*      \n" +
"vy8dw9@@@@@@@@@@@@@@@Q:                  `M@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@Q;     \n" +
"      $@?;tHB@@@@@@@@@B*`               'k@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@<    \n" +
"      m^?^;'yg@@@@@@@@@@1             `v@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@X'  \n" +
"      I   '=\\`R@@@@@@@@@@;          ,wB@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@N  \n" +
"      1       `#@@@@@@@@@@9?'`  ';IO@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@' \n" +
"      t        .H@@@@@@@@@@@@@BB@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@; \n" +
"      }         `N@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@: \n" +
"      }          `p@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@` \n" +
"      ;?           *%@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@BR@@@@@@@@@@@@@B  \n" +
"       :'            ~yB@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@U?`?@@@@@@@@@@@@@B` \n" +
"                        ,iXDB@@@@@@B#qy7^,1B@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@BI,   ;@@@@@@@@@@@@@@^ \n" +
"                                           `\\N@@@@@@@@@@@@@@@@@@@@@@@@@@Bpy=.      I@@@@@@@@@@@@@@B.\n" +
"                                              Q@@@@@@r/}}1i+?|iiii|?r;,`           'B@@@@@@@@@@@@B@,\n" +
"                                              ?@@@@@@                               :B@@@@@@@@@@@GH \n" +
"                                              `d@@@@@,                               ,B@@@@@@@@@@>; \n" +
"                                               ;@@@@@=                                r@@@@@@@@@@r  \n" +
"                                                Q@@@@y                                ^@@@@qp@@@@I  \n" +
"                                                X@@@@N`                               ;@@@@Dr@@@@B` \n" +
"                                                l@@@@@<                               '@@@@B G@@@@i \n" +
"                                                +@@@@@R                                B@@@@`'@@@@B`\n" +
"                                                |@@@@@@~                               @@@@@* A@@@@R\n" +
"                                                I@@@@@@.                              ,@@@@B~ A@@@@l\n" +
"                                                ;@@@@@@                               j@@@#'  $@@@h \n" +
"                                                 k@@@@N                               B@@R`   y@@@. \n" +
"                                                 ?@@R@R                              :@@q.    z@@t  \n" +
"                                                 *@B*@g                             `R@D`     w@N`  \n" +
"                                                 i@W;@B                             }@B.      8@7   \n" +
"                                                 w@$;@@                            :@@:      `@@`   \n" +
"                                                 B@Xy@@:                          :B@|       <@q    \n" +
"                                                f@@D@@@:                         +@@j       `R@1    \n" +
"                                               :@@@@@@B.                    `;|yB@@v`       W@@?    \n" +
"                                            `:FB@@@@BBW^                    :GpDWgp      `:$@@H`    \n" +
"                                           ~Q@@@@@#i                                   `D@@@@@t     \n\n\n" +
"                                        ####################                                        \n" +
"                                        ##### CamelBot #####                                        \n" +
"                                        ####################                                        \n"

console.log(splashScreen)

const winston = require('winston');
const Discord = require('discord.js');
const camelLibjs = require('./camelLib');
const plugClass = require('./plugClass');
const commandClass = require('./command');
const commandRunner = require('./commandRunner');
const fs = require('fs');
const mappedClass = require('./mappedClass');
const camellib = new camelLibjs({

    'private': require('./configs/private.json'),
    'database': require('./configs/database.json'),
});

// This is the main logger that core code uses
const logger = winston.createLogger({
    level: 'info',
    format: winston.format.combine(
        winston.format.timestamp({
            format: 'YYYY-MM-DD HH:mm:ss'
        }),
        winston.format.errors({
            stack: true
        }),
        winston.format.splat(),
        winston.format.json()
    ),
    defaultMeta: {
        service: 'CamelBot Core'
    },
    transports: [
        //
        // - Write to all logs with level `info` and below to `quick-start-combined.log`.
        // - Write all logs error (and below) to `quick-start-error.log`.
        //
        new winston.transports.File({
            filename: './logs/error.log',
            level: 'error'
        }),
        new winston.transports.File({
            filename: './logs/logger.log'
        })
    ]
});

// This logs to the console
logger.add(new winston.transports.Console({
    format: winston.format.combine(
        winston.format.colorize(),
        winston.format.simple()
    )
}));



camellib.logger = logger;

let client = new Discord.Client({
    partials: ['MESSAGE', 'CHANNEL', 'REACTION'],
    intents: [Discord.Intents.FLAGS.GUILDS, Discord.Intents.FLAGS.GUILD_MESSAGES, Discord.Intents.FLAGS.GUILD_MEMBERS]
});

// Log into the Discord API
client.login(camellib.private.token).catch((error) => {
    logger.error('Unable to initialize bot, check your key and connection.\n');
    logger.error(error);
}).then(() => {
    camellib.client = client;





// Once Discord is ready
client.on('ready', async () => {
    client.guilds.cache.forEach(guild => {
        // Create an entry in the database for each guild
        if (!camellib.database.has(guild.id)) {
            camellib.database.set(guild.id, {
                'id': guild.id,
                'name': guild.name,
                'enabledPlugins': [],
            });
        } else {
            try {
                camellib.database.get(guild.id).name = guild.name;
            } catch (err) {
                logger.error('Error while updating guild information');
                logger.error(err);

            }

        }

    });

    // Save the database after a possible write
    camellib.saveDatabase();
    // Map the commands for loading later
    camellib.mapCoreCommands();
    // There may be unused commands in the API, remove them
    camellib.purgeCommands();
    // Make sure that all commands are up to date on each guild
    camellib.publishCommands();
    // That way everyone knows that plugins are loaded so they can make callbacks
    camellib.emit('pluginsLoaded');
});




// Initialize plugins with manifests
getDirectories('./plugins').toString().split(',').forEach(element => {
    if (fs.readdirSync('./plugins/' + element).includes('manifest.json')) {
        try {
            // Load in the manifest
            let tempManifest = require('./plugins/' + element + '/manifest.json');
            // Load the class of the plugin
            let tempObject = require('./plugins/' + element + '/' + tempManifest.class);

            // Create a logger for the plugin's class so we don't clog the main logger
            let tempLogger = winston.createLogger({
                level: 'info',
                format: winston.format.combine(
                    winston.format.timestamp({
                        format: 'YYYY-MM-DD HH:mm:ss'
                    }),
                    winston.format.errors({
                        stack: true
                    }),
                    winston.format.splat(),
                    winston.format.json()
                ),
                defaultMeta: {
                    service: tempManifest.name
                },
                transports: [
                    //
                    // - Write to all logs with level `info` and below to `quick-start-combined.log`.
                    // - Write all logs error (and below) to `quick-start-error.log`.
                    //
                    new winston.transports.File({
                        filename: './logs/error.log',
                        level: 'error'
                    }),
                    new winston.transports.File({
                        filename: './logs/logger.log'
                    })
                ]
            });
            tempLogger.add(new winston.transports.Console({
                format: winston.format.combine(
                    winston.format.colorize(),
                    winston.format.simple()
                )
            }));


            // Map the plugin's main class
            camellib.mappedClasses.set(element + '/' + tempManifest.class, new tempObject(new mappedClass(tempLogger, camellib)));

            camellib.plugins.set(tempManifest.name, new plugClass(camellib.mappedClasses.get(element + '/' + tempManifest.class), [], tempManifest));

            tempManifest.commands.forEach(command => {
                // Load the command's class
                let tempObject = require('./plugins/' + element + '/' + command.class);
                // Map the command's class
                if (!camellib.mappedClasses.has(element + '/' + command.class)) {
                    camellib.mappedClasses.set(element + '/' + command.class, new tempObject());
                }
                // Map the command
                camellib.mappedCommands.set(command.name, new commandClass(command, camellib.mappedClasses.get(element + '/' + tempManifest.class)[command.method], tempManifest.name));
                // Push them to the plugin's object so the command loader knows what to load
                camellib.plugins.get(tempManifest.name).commands.push(command.name);
            });
        } catch (err) {
            logger.error('Unable to load manifest for plugin ' + element + ': ' + err);
        }
    } else {
        if (!fs.readdirSync('./plugins/' + element).includes('ignore')) {
            logger.error('Plugin ' + element + ' has no manifest, skipping loading.');
        }
    }
});


// That way everyone knows that plugins are loaded so they can make callbacks
camellib.emit('pluginsLoaded');

client.on('interaction', interaction => {
    if (interaction.isCommand()) {
        // Super easy way to call methods that are already mapped
        camellib.mappedCommands.get(interaction.command.name).method(new commandRunner(interaction, null, 'discord'));
    }
});

/**
 * 
 * @param {String} path Path of the folder you want to get a list of files from
 * @returns {null} 
 */
function getDirectories(path) {
    return fs.readdirSync(path).filter(function(file) {
        return fs.statSync(path + '/' + file).isDirectory();
    });

}

// Uh oh danger time
// Hot loading new servers so we don't break stuff
client.on('guildCreate',guild=>{
    camellib.database.set(guild.id,{
        "id":guild.id,
        "name":guild.name,
        "enabledPlugins":[]
    });
    camellib.saveDatabase();
    camellib.emit("guildJoined",guild);
});

client.on('guildDelete',guild=>{
    camellib.database.delete(guild.id);
    camellib.saveDatabase();
    camellib.emit("guildKicked",guild);
})

