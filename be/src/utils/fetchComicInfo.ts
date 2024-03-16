import { Comic as PrismaComic } from "@prisma/client";
import * as cheerio from 'cheerio'; // Assuming cheerio for parsing
import { prisma } from "../db";

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

export function parseComicHtmlPage(html: string, url: string): Comic {
    const $ = cheerio.load(html); // Parse HTML with Cheerio
    const result: Comic = {
        pythonFetchInfo: true,
    };
    if (url.includes("blogtruyenmoi.com")) {
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
    } else if (url.includes("nettruyenee.com")) {
        const thumbnail = $("#item-detail > div.detail-info > div > div.col-xs-4.col-image > img")
        if (thumbnail.length) {
            result.thumbnail = thumbnail.attr('src');
        }
        const status = $("#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.status.row > p.col-xs-8");
        if (status.length) {
            result.status = status.text().trim();
        }
        const genre = $("#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.kind.row > p.col-xs-8")
        if (genre.length) {
            result.genre = {}
            genre.find('a').each((_, link) => {
                result.genre![$(link).text().trim()] = $(link).attr('href') || '';
            });
        }
        const content = $("#item-detail > div.detail-content > p")
        if (content.length) {
            result.content = content.text().trim();
        }
        const author = $("#item-detail > div.detail-info > div > div.col-xs-8.col-info > ul > li.author.row > p.col-xs-8")
        if (author.length) {
            result.author = {}
            const a_element = author.find('a')
            if (a_element.length) a_element.each((_, link) => {
                result.author![$(link).text().trim()] = $(link).attr('href') || '';
            })
            else author.text().trim().split(';').forEach((author) => {
                result.author![author.trim()] = '';
            })
        }
    }


    return result;
}

export function processArrayComic(comic: PrismaComic[]) {
    return Promise.all(comic.map(async (c) => {
        if (!c.thumbnail) {
            // refetch comic update in db and return
            const resp = await (await fetch(c.url)).text()
            const parsed = (parseComicHtmlPage(resp, c.url))
            return await prisma.comic.update({
                where: {
                    id: c.id
                },
                data: parsed
            })
        }
        return c
    }))
}