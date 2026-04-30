import { z } from "zod"

// Custom dictionary
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

// Warframe v2 Base response
const BaseResponseSchema = z.object({
  apiVersion: z.string(),
  data: z.unknown(),
  error: z.null()
})

// Item i18n details
const ItemI18NSchema = z.looseObject({
  name: z.string(),
  description: z.string().optional(),
  wikiLink: z.string().optional(),
  icon: z.string(),
  thumb: z.string(),
  subIcon: z.string().optional()
})

// Item details
export const ItemSchema = z.looseObject({
  id: z.string(),
  slug: z.string(),
  gameRef: z.string(),
  tags: z.array(z.string()).optional(),
  setRoot: z.boolean().optional(),
  setParts: z.array(z.string()).optional(),
  quantityInSet: z.number().int().optional(),
  rarity: z.string().optional(),
  bulkTradable: z.boolean().optional(),
  subtypes: z.array(z.string()).optional(),
  maxRank: z.number().int().optional(),
  maxCharges: z.number().int().optional(),
  maxAmberStars: z.number().int().optional(),
  maxCyanStars: z.number().int().optional(),
  baseEndo: z.number().int().optional(),
  endoMultiplier: z.number().optional(),
  ducats: z.number().int().optional(),
  vosfor: z.number().int().optional(),
  reqMasteryRank: z.number().int().optional(),
  vaulted: z.boolean().optional(),
  tradingTax: z.number().int().optional(),
  tradable: z.boolean().optional(),
  i18n: z.record(z.string(), ItemI18NSchema).optional()
})

// get-item response
export const GetItemResponseSchema = BaseResponseSchema.extend({
  data: ItemSchema
})

// Statistics
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
});

// Orders
export const OrderGroupByProperties = z.enum(["itemId", "rank", "charges", "subtype", "amberStars", "cyanStars"])

export const OrderSchema = z.looseObject({
  id: z.string(),
  type: z.enum(["buy", "sell"]),
  platinum: z.number().int(),
  quantity: z.number().int(),
  perTrade: z.number().int().optional(),
  rank: z.number().int().optional(),
  charges: z.number().int().optional(),
  subtype: z.string().optional(),
  amberStars: z.number().int().optional(),
  cyanStars: z.number().int().optional(),
  visible: z.boolean(),
  createdAt: z.string(),
  updatedAt: z.string(),
  itemId: z.string(),
  group: z.string().optional()
})

// User details
export const UserSchema = z.looseObject({
  id: z.string(),
  ingameName: z.string(),
  avatar: z.string().optional(),
  background: z.string().optional(),
  about: z.string().optional(),
  reputation: z.number().int(),
  masteryLevel: z.number().int().optional(),

  platform: z.string(),
  crossplay: z.boolean(),
  locale: z.string(),

  status: z.string(),
  activity: z.object({ type: z.string(), details: z.string() }),
  lastSeen: z.string(),
})

// Order with user details
export const OrderWithUserSchema = OrderSchema.extend({
  user: UserSchema
})

// get-orders response
export const GetOrdersResponseSchema = BaseResponseSchema.extend({
  data: z.array(OrderWithUserSchema)
})