import { json } from '@sveltejs/kit';
import { GetOrdersResponseSchema } from '$lib/schemas';

export async function GET({ params }): Promise<Response> {
  const { slug } = params;
  const response = await fetch(`https://api.warframe.market/v2/orders/item/${slug}`);
  const data = await response.json();

  const result = GetOrdersResponseSchema.safeParse(data);

  if (!result.success) {
    return new Response(JSON.stringify(result.error.issues), { status: 400 });
  }

  return json(result.data);
}