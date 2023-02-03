// Next.js API route support: https://nextjs.org/docs/api-routes/introduction
import type { NextApiRequest, NextApiResponse } from "next";

type Article = {
  id: string;
  title: string;
  author: string;
  media: string;
  url: string;
  summary: string;
  created_at: string;
  crawled_at: string;
};

export default function handler(
  req: NextApiRequest,
  res: NextApiResponse<Data>
) {
  res.status(200).json({ name: "John Doe" });
}
