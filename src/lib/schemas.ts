import { z } from "zod"

export const DictionarySchema = z.object({
  last_fetched_at: z.string(),
  tradeable_items: z.array(
    z.object({
      slug: z.string(),
      name: z.string(),
      tags: z.array(z.string()),
      ducats: z.optional(z.number()),
      vaulted: z.optional(z.boolean())
    })
  )
})

export const ItemSchema = z.looseObject({
  apiVersion: z.string(),
  data: z.object({
    id: z.string(),
    slug: z.string(),
    gameRef: z.string(),
    tags: z.array(z.string()),
    setRoot: z.optional(z.boolean()),
    setParts: z.optional(z.array(z.string())),
    quantityInSet: z.optional(z.number()),
    ducats: z.optional(z.number()),
    reqMasteryRank: z.optional(z.number()),
    tradingTax: z.number(),
    tradable: z.boolean(),
    i18n: z.object({
      en: z.object({
        name: z.string(),
        description: z.string(),
        icon: z.string(),
        thumb: z.string(),
        subIcon: z.optional(z.string())
      })
    })
  }),
  error: z.null()
})

const StatisticBase = z.object({
  datetime: z.string(),
  volume: z.number(),
  min_price: z.number(),
  max_price: z.number(),
  avg_price: z.number(),
  wa_price: z.number(),
  median: z.number(),
  moving_avg: z.optional(z.number()),
  id: z.string()
})

export const ClosedStatisticItem = z.looseObject({
  ...StatisticBase.shape,
  open_price: z.number(),
  closed_price: z.number(),
  donch_top: z.number(),
  donch_bot: z.number(),
})

export const LiveStatisticItem = z.looseObject({
  ...StatisticBase.shape,
  order_type: z.string()
})

export const StatisticsSchema = z.object({
  payload: z.object({
    statistics_closed: z.object({
      "48hours": z.array(ClosedStatisticItem),
      "90days": z.array(ClosedStatisticItem)
    }),
    statistics_live: z.object({
      "48hours": z.array(LiveStatisticItem),
      "90days": z.array(LiveStatisticItem)
    })
  })
})
