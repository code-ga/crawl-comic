import { Elysia, t } from "elysia";
import prisma from "../db";
import { Genre } from "@prisma/client";

export const apiRouter = new Elysia({
  prefix: "/api",
})
  // comics
  .get(
    "/comics/id",
    async ({ query }) => {
      console.log(`comics/id: ${query}`);
      return await prisma.comic.findUnique({
        where: {
          id: query.id,
        },
      });
    },
    {
      query: t.Object({
        id: t.String({
          minLength: 1,
        }),
      }),
    }
  )
  // tạm thời thay thế search system
  .get(
    "/comics/name",
    async ({ query }) => {
      console.log(`comics/name: ${query}`);
      query.name.split(" ").join(" & ");

      const name = await prisma.comic.findMany({
        where: {
          name: {
            contains: query.name,
          },
        },
      });

      const alias = await prisma.comic.findMany({
        where: {
          aliases: {
            hasSome: query.name.split(" "),
          },
        },
      });

      return name.reduce((acc, val) => {
        if (!acc.includes(val)) {
          acc.push(val);
        }
        return acc;
      }, alias);
    },
    {
      query: t.Object({
        name: t.String({
          minLength: 1,
        }),
      }),
    }
  )
  // recommend
  .get(
    "/recommend",
    async ({ query }) => {
      console.log(`recommend: ${query}`);

      return await prisma.comic.findMany({
        // where: {
        //   likes: {
        //     gt: query.min ?? 1,
        //   },
        // },
        orderBy: {
          likes: "desc",
        },
        take: query.quantity,
      });
    },
    {
      query: t.Object({
        min: t.Optional(
          t.Numeric({
            minimum: 1,
          })
        ),
        quantity: t.Numeric({
          default: 10,
          minimum: 2,
        }),
      }),
    }
  )
  // add comic
  .post(
    "/dashboard",
    async ({ body }) => {
      console.log(body.name);
      const c = await prisma.comic
        .create({
          data: {
            name: body.name,
            aliases: body.aliases,
            thumbnail: body.thumbnail,
            description: body.description,
            genre: body.genre,
            color: body.color,
          },
        })
        .catch(console.error);
      console.log(c);
      return c;
    },
    {
      body: t.Object({
        name: t.String(),
        aliases: t.Array(t.String()),
        thumbnail: t.String(),
        description: t.String(),
        genre: t.Array(t.Enum(Genre)),
        color: t.BooleanString(),
      }),
    }
  );
