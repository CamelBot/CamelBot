// CamelBot 4, but this time he's mad
// Let's try not to hate everything edition
// jkcoxson

console.log("System Startup, all hail camels")

// Narcisism
let splashScreen = "\n"+  
"                               ./#%#########%%(,                                \n"+
"                         .*////(#%#########%%%(////,                            \n"+
"                       ,/#%%%%%##########%%&&&&%%%%#(/(/,                       \n"+
"                    .*/#%%##############%%&&%%#######%%%#(*,                    \n"+
"                    *#%%##%%%#####%%#####%%%#######%%%##%%#*                    \n"+
"                    ,(###%&&%##%#(//(########%%%###%&&%###(*                    \n"+
"                    *#%%%%%%%###/*,*(%%%%%%#((//(##%&&%%%%#*                    \n"+
"                    .*(%&&%###(,...,*//////*,. .,/#%%&&&%(*.                    \n"+
"                       *(#((#%%%####((/,,*/(#####%%#((((*                       \n"+
"                    .,*,.  ,#@@&%##%@@%**#@@&##%&@@%*. .,*,.                    \n"+
"                    *(%%#((#&@%.   *%@&##%@&/.  .(@@%(((%%#*                    \n"+
"                    ,(%&&%%&@@&(/(&@@@@@@@@@@&#/(%@@&%%&&&#*                    \n"+
"                    ,//*. .,(%&&&&%%%%#((#%%%%&&&&%(,. .*//,                    \n"+
"                       .***,,,,..,,...*((/,..,,..,,,,***,                       \n"+
"                      .*//*,,,**,**,**(#(/*,,*****,,,*//*.                      \n"+
"                      .,//*,,,,,,,,,/(#((*,,,,,,,,,,,*//*.                      \n"+
"                      .*//*,,,,,,***,,.,,,,,***,,,,,,*//*.                      \n"+
"                      .,//*,,,,,,/((/******/((/*,,,,,*//*.                      \n"+
"                         .*//,,,,,,,/(#((#(/*,,,,,,*/*.                         \n"+
"                   ......,/(((/*****,,,,,,,,*****/(((/,......                   \n"+
"            ..   *#@@@@@@@@@@@@@@@@&(*,,,,*(&@@@@@@@@@@@@@@@@%*.  ..            \n"+
"         .(&@@@@@&%#############%%&&&&&&&&&&&&%%#############%&@@@@@@#,         \n"+
"         .(@@@&&&&%#############%%&&&&&&&&&&&&%%#############%%&&&&@@#,         \n"+
"         ,(@@&&%&%%#############%%%%%%%%%%%%%%%%#############%%&%%&@@#,         \n"+
"    ,#@@@@@@@&&&&%%##########################################%%&&&&@@@@@@@%*    \n"+
"    ./##(///((#####################################################(///(##(,    \n"+
"    ./((/***/((((#%&%%####################################%%&%##(#(/***/((/,    \n"+
"    ./##(/*/((#(##%&&%##%#################################%&&%##(#((/**/(#/,    \n"+
"    ./((*,,,/(####%&&%####################################%%&%##(#(/*,,*((/,    \n"+
"    ./((*,.,*(####%&%%####################################%&&%####(/,..*((/,    \n"+
"    ./((*,.,/(####%&&&%%%#################################%%&%##(#(/,.,*((/,    \n"+
"    ./((*,.,*(####%&&&&%%##%%##########################%%%&&&%####(/,..*((/,    \n"+
"\n\n"+
"                           ########################                             \n"+
"                           ###     CamelBot     ###                             \n"+
"                           ###   by jkcoxson    ###                             \n"+
"                           ########################                             \n\n"

console.log(splashScreen)

const winston = require('winston');
const Discord = require('discord.js')
const camelLibjs = require('./camelLib');
const plugClass = require('./plugClass')
const commandClass = require('./command')
const commandRunner = require('./commandRunner')
const fs = require('fs');
const { cli } = require('winston/lib/winston/config');
const { callbackify } = require('util');
const mappedClass = require('./mappedClass');
const camellib = new camelLibjs({
    "private":require('./configs/private.json'),
    "database":require('./configs/database.json'),
})


const logger = winston.createLogger({
    level: 'info',
    format: winston.format.combine(
        winston.format.timestamp({
            format: 'YYYY-MM-DD HH:mm:ss'
        }),
        winston.format.errors({ stack: true }),
        winston.format.splat(),
        winston.format.json()
    ),
    defaultMeta: { service: 'CamelBot Core' },
    transports: [
        //
        // - Write to all logs with level `info` and below to `quick-start-combined.log`.
        // - Write all logs error (and below) to `quick-start-error.log`.
        //
        new winston.transports.File({ filename: './logs/error.log', level: 'error' }),
        new winston.transports.File({ filename: './logs/logger.log' })
    ]
});
  
logger.add(new winston.transports.Console({
    format: winston.format.combine(
        winston.format.colorize(),
        winston.format.simple()
    )
}));



camellib.logger = logger

let client = new Discord.Client({ partials: ['MESSAGE', 'CHANNEL', 'REACTION'], intents: [Discord.Intents.FLAGS.GUILDS, Discord.Intents.FLAGS.GUILD_MESSAGES,Discord.Intents.FLAGS.GUILD_MEMBERS] });

client.login(camellib.private.token).catch((error)=>{
    logger.error("Unable to initialize bot, check your key and connection.\n")
    logger.error(error)
}).then(()=>{
    camellib.client=client
    
});

client.on('ready',async ()=>{
    client.guilds.cache.forEach(guild=>{
        if(!camellib.database.has(guild.id)){
            camellib.database.set(guild.id,{
                "id":guild.id,
                "name":guild.name,
                "enabledPlugins":[],
            })
        }else{
            try{
                camellib.database.get(guild.id).name=guild.name
            }catch(err){
                logger.error("Error while updating guild information")
                logger.error(err)
            }
            
        }
    })
    camellib.saveDatabase();
    camellib.mapCoreCommands();
    camellib.purgeCommands();
    camellib.publishCommands();
    
})



// Initialize plugins with manifests
getDirectories('./plugins').toString().split(',').forEach(element=>{
    if (fs.readdirSync('./plugins/'+element).includes('manifest.json')){
        try{
            // Load in the manifest
            let tempManifest = require('./plugins/'+element+"/manifest.json");
            // Load the class of the plugin
            let tempObject = require('./plugins/'+element+"/"+tempManifest.class)
            // Create a logger for the plugin's class
            let tempLogger = winston.createLogger({
                level: 'info',
                format: winston.format.combine(
                    winston.format.timestamp({
                        format: 'YYYY-MM-DD HH:mm:ss'
                    }),
                    winston.format.errors({ stack: true }),
                    winston.format.splat(),
                    winston.format.json()
                ),
                defaultMeta: { service: tempManifest.name },
                transports: [
                    //
                    // - Write to all logs with level `info` and below to `quick-start-combined.log`.
                    // - Write all logs error (and below) to `quick-start-error.log`.
                    //
                    new winston.transports.File({ filename: './logs/error.log', level: 'error' }),
                    new winston.transports.File({ filename: './logs/logger.log' })
                ]
            })
            tempLogger.add(new winston.transports.Console({
                format: winston.format.combine(
                    winston.format.colorize(),
                    winston.format.simple()
                )
            }));
            // Define an object for each plugin
            
            // Map the plugin's main class
            camellib.mappedClasses.set(element+"/"+tempManifest.class,new tempObject(new mappedClass(tempLogger,camellib)));
<<<<<<< HEAD
            camellib.plugins.set(tempManifest.name,new plugClass(camellib.mappedClasses.get(element+"/"+tempManifest.class),[],tempManifest));
=======
            camellib.plugins.set(tempManifest.name,new plugClass(camellib.mappedClasses.get(element+"/"+tempManifest.class),[],tempManifest.name,tempManifest.description));
>>>>>>> b48662d013d1ad3ec0bf0c83ff855c8c69463047
            tempManifest.commands.forEach(command=>{
                // Load the command's class
                let tempObject = require('./plugins/'+element+"/"+command.class)
                // Map the command's class
                if(!camellib.mappedClasses.has(element+"/"+command.class)){
                    camellib.mappedClasses.set(element+"/"+command.class,new tempObject())
                }
                // Map the command
                camellib.mappedCommands.set(command.name,new commandClass(command,camellib.mappedClasses.get(element+"/"+tempManifest.class)[command.method],tempManifest.name));
                // Push them to the plugin's object so the command loader knows what to load
                camellib.plugins.get(tempManifest.name).commands.push(command.name);
            })
        }catch(err){
            logger.error("Unable to load manifest for plugin "+element+": "+err)
        }
    }else{
        if (!fs.readdirSync('./plugins/'+element).includes('ignore')){
            logger.error("Plugin "+element+" has no manifest, skipping loading.")
        }
    }
})

camellib.emit('pluginsLoaded');

client.on('interaction',interaction=>{
    if(interaction.isCommand()){
        camellib.mappedCommands.get(interaction.command.name).method(new commandRunner(interaction,null,'discord'))
    }
})

function getDirectories(path) {
    return fs.readdirSync(path).filter(function (file) {
      return fs.statSync(path+'/'+file).isDirectory();
    });
}

