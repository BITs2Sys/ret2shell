import type { DateTime } from "luxon";

export type Hint = {
  id: number;
  created_at: DateTime;
  challenge_id: number;
  content: string;
  cost: number;
};
