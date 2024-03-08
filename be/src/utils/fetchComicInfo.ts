import * as cheerio from 'cheerio'; // Assuming cheerio for parsing

interface Comic {
    author?: { [name: string]: string };
    source?: { [name: string]: string };
    translatorTeam?: { [name: string]: string };
    postedBy?: { [name: string]: string };
    genre?: { [name: string]: string };
    content?: string;
    thumbnail?: string;
    anotherName?: string[];
    status?: string;
    pythonFetchInfo?: boolean;
}

export function parseComicHtmlPage(html: string): Comic {
    const $ = cheerio.load(html); // Parse HTML with Cheerio

    const result: Comic = {
        pythonFetchInfo: true,
    };

    // Fetch description
    const contentTag = $('#wrapper > section.main-content > div > div.col-md-8 > section > div.detail > div.content');
    if (contentTag.length) {
        result.content = contentTag.text().trim();
    }

    const thumbnail = $('#wrapper > section.main-content > div > div.col-md-8 > section > div.thumbnail > img');
    if (thumbnail.length) {
        result.thumbnail = thumbnail.attr('src');
    }

    // Fetch other information
    const description = $('#wrapper > section.main-content > div > div.col-md-8 > section > div.description');
    if (description.length && description.find('p').length) {
        description.find('p').each((_, child) => {
            const childText = $(child).text().trim();
            if (childText.startsWith('Tên khác:')) {
                result.anotherName = childText
                    .slice(childText.indexOf(':') + 1)
                    .trim()
                    .split(';');
            } else if (childText.startsWith('Tác giả:')) {
                const authorLink = $(child).find('a');
                result.author = { [authorLink.text().trim()]: authorLink.attr('href') || '' };
            } else if (childText.startsWith('Nguồn:')) {
                const sourceLink = $(child).find('a');
                result.source = { [sourceLink.text().trim()]: sourceLink.attr('href') || '' };
            } else if (childText.startsWith('Nhóm dịch:')) {
                const translatorLinks = $(child).find('a');
                result.translatorTeam = {};
                translatorLinks.each((_, link) => {
                    result.translatorTeam![$(link).text().trim()] = $(link).attr('href') || '';
                });
            } else if (childText.startsWith('Đăng bởi:')) {
                const postedByLink = $(child).find('a');
                result.postedBy = { [postedByLink.text().trim()]: postedByLink.attr('href') || '' };
                const statusSpan = $(child).find('span');
                if (statusSpan.length) {
                    result.status = statusSpan.text().trim();
                }
            } else if (childText.startsWith('Thể loại:')) {
                const genreLinks = $(child).find('a');
                result.genre = {};
                genreLinks.each((_, link) => {
                    result.genre![$(link).text().trim()] = $(link).attr('href') || '';
                });
            }
        });
    }

    return result;
}