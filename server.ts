import * as http from 'http';
import { markdownEngine as markdown, htmlTag } from 'discord-markdown';

const server = http.createServer(async (req, res) => {
    console.log('ok');

    const url = req.url?.split('?')[0];
    const bodyStr = await new Promise<string>((res, rej) => {
        let bodyStr = '';
        req.on('data', (chunk) => {
            bodyStr += chunk
        })
        req.on('end', () => {
            res(bodyStr);
        })
    })
    console.log(bodyStr);
    if (url === '/parse-discord-markdown') {
        const data = parseDiscrodMarkdown(bodyStr);
        console.log(data);
        res.setHeader('Content-Type', 'application/json')
        res.end(JSON.stringify(data));
        return;
    }
})

function parseDiscrodMarkdown(message: string): any {
    const ast = markdown.parserFor({
        atDC: bridgeRule.atDC,
        atQQ: bridgeRule.atQQ,
        atKHL: bridgeRule.atKHL,
        discordUser: bridgeRule.discordUser,
        discordEveryone: bridgeRule.discordEveryone,
        discordHere: bridgeRule.discordHere,
        DiscordEmoji: bridgeRule.DiscordEmoji,
        Plain: bridgeRule.Plain,
    })(message) as Array<{ type: 'discordUser' | 'discordEmoji', [prop: string]: any }>;
    return ast;
}

export const bridgeRule = {
    atDC: {
        order: 0,
        match: source => /^@\[DC\] [^\n]+?#\d\d\d\d/.exec(source),
        parse: function (capture, parse, state) {
            console.log(capture);
            return { type: 'At', username: capture[0] };
        },
        html: function (node, output, state) {
            return '{{atDc}}';
        },
    },
    atKHL: {
        order: 0,
        match: source => /^@\[KHL\] ([^\n#]+)#(\d\d\d\d)/.exec(source),
        parse: function (capture, parse, state) {
            console.log(capture);
            return { type: 'At', source: 'KHL', username: capture[1], discriminator: capture[2] };
        },
        html: function (node, output, state) {
            return '{{atDc}}';
        },
    },
    atQQ: {
        order: 0,
        match: source => /^@\[QQ\] [^\n]+?\([0-9]+\)/.exec(source),
        parse: function (capture, parse, state) {
            console.log(capture);
            return { type: 'At', username: capture[0] };
        },
        html: function (node, output, state) {
            return '{{atDc}}';
        },
    },
    Plain: Object.assign({}, markdown.defaultRules.text, {
        match: source => /^[\s\S]+?(?=[^0-9A-Za-z\s\u00c0-\uffff-]|\n\n|\n|\w+:\S|$)/.exec(source),
        parse: function (capture, parse, state) {
            return { type: 'Plain', text: capture[0] };
        },
        html: function (node, output, state) {
            if (state.escapeHTML)
                return markdown.sanitizeText(node.content);

            return node.content;
        },
    }),
    discordUser: {
        order: markdown.defaultRules.strong.order,
        match: source => /^<@!?([0-9]*)>/.exec(source),
        parse: function (capture) {
            return {
                id: capture[1]
            };
        },
        html: function (node, output, state) {
            return htmlTag('span', state.discordCallback.user(node), { class: 'd-mention d-user' }, state);
        }
    },
    discordEveryone: {
        order: markdown.defaultRules.strong.order,
        match: source => /^@everyone/.exec(source),
        parse: function () {
            return { type: 'AtAll' };
        },
        html: function (node, output, state) {
            return htmlTag('span', state.discordCallback.everyone(node), { class: 'd-mention d-user' }, state);
        },
    },
    discordHere: {
        order: markdown.defaultRules.strong.order,
        match: source => /^@here/.exec(source),
        parse: function () {
            return { type: 'AtAll' };
        },
        html: function (node, output, state) {
            return htmlTag('span', state.discordCallback.here(node), { class: 'd-mention d-user' }, state);
        }
    },
    DiscordEmoji: {
        order: markdown.defaultRules.strong.order,
        match: source => /^<(a?):(\w+):(\d+)>/.exec(source),
        parse: function (capture) {
            return {
                animated: capture[1] === "a",
                name: capture[2],
                id: capture[3],
            };
        },
        html: function (node, output, state) {
            return htmlTag('img', '', {
                class: `d-emoji${node.animated ? ' d-emoji-animated' : ''}`,
                src: `https://cdn.discordapp.com/emojis/${node.id}.${node.animated ? 'gif' : 'png'}`,
                alt: `:${node.name}:`
            }, false, state);
        }
    },
    khlEveryone: {
        order: markdown.defaultRules.strong.order,
        match: source => /\(met\)all\(met\)/.exec(source),
        parse: function () {
            return { type: 'AtAll' };
        },
        html: function (node, output, state) {
            return htmlTag('span', state.discordCallback.everyone(node), { class: 'd-mention d-user' }, state);
        },
    },
};

parseDiscrodMarkdown(`@[DC] 6uopdong#4700
!绑定 qq 1261972160 asd`);
server.listen(3000)