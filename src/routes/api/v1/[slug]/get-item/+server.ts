// src/routes/api/get-data/+server.js
import { json } from '@sveltejs/kit';
import { z } from 'zod';
import { ItemSchema } from '$lib/schemas';

export async function GET({ params }): Promise<Response> {
  const { slug } = params;
  const response = await fetch(`https://api.warframe.market/v2/item/${slug}`);
  const data = await response.json();

  const result = ItemSchema.safeParse(data);

  if (!result.success) {
    return new Response(JSON.stringify(result.error.issues), { status: 400 });
  }

  return json(result.data);
}