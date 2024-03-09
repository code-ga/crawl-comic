import { Elysia, t } from "elysia";
import prisma from "../db";

export const apiRouter = new Elysia({
  prefix: "/api",
  name: "Api routing",
})
  // comics
  .get(
    "/comics/id",
    async ({ query }) => {
      console.log(query);
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
  .get(
    "/comics/name",
    async ({ query }) => {
      console.log(`comics/name: ${query}`);

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
      }, alias)
    },
    {
      query: t.Object({
        name: t.String({
          minLength: 1,
        }),
      }),
    }
  );
