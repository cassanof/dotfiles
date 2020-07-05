const { Plugin } = require('powercord/entities');

const Constants = Object.freeze({
  SEARCH_ENGINES: {
    google: 'g',
    yahoo: 'y',
    bing: 'b',
    ask: 'k',
    aol: 'a',
    duckduckgo: 'd'
  },
  SEARCH_TYPES: {
    web: 'w',
    images: 'i',
    videos: 'v',
    news: 'n',
    shopping: 's'
  }
});

module.exports = class LMGTFY extends Plugin {
  startPlugin () {
    powercord.api.commands.registerCommand({
      command: 'lmgtfy',
      description: 'Let me Google that for you...',
      usage: '{c} [...search terms] <search engine> <search type>',
      executor: this.handleCommand.bind(this)
    });
  }

  pluginWillUnload () {
    powercord.api.commands.unregisterCommand('lmgtfy');
  }

  handleCommand (args) {
    if (args.length < 1) {
      return;
    }

    const options = args.slice(-2).map(arg => arg.toLowerCase());
    const params = {};
    params.searchEngine = Constants.SEARCH_ENGINES.google;

    for (const key of Object.keys(Constants)) {
      if (key === 'SEARCH_ENGINES') {
        for (const searchEngine of Object.keys(Constants[key])) {
          for (let i = 0; i < options.length; i++) {
            const match = options[i].toLowerCase() === searchEngine;
            if (match) {
              params.searchEngine = Constants.SEARCH_ENGINES[searchEngine];
              args.splice(args.indexOf(searchEngine), 1);
              options.splice(i, 1);
              break;
            }
          }
        }
      } else if (key === 'SEARCH_TYPES') {
        for (const searchType of Object.keys(Constants[key])) {
          for (let i = 0; i < options.length; i++) {
            const match = options[i].toLowerCase() === searchType;
            if (match) {
              if (params.searchEngine === 'g') {
                params.searchType = Constants.SEARCH_TYPES[searchType];
                args.splice(args.indexOf(searchType), 1);
              }
            }
          }
        }
      }
    }

    const { searchEngine, searchType } = params;
    const queryString = new URLSearchParams();
    queryString.append('q', args.join(' '));
    if (searchType) {
      queryString.append('s', searchEngine);
      queryString.append('t', searchType);
    } else if (searchEngine !== 'g') {
      queryString.append('s', searchEngine);
    }

    return {
      send: true,
      result: `<https://lmgtfy.com/?${queryString}>`
    };
  }
};
