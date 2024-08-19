import xdsecMascot01 from "./xdsec-mascot-01.webp";
import xdsecMascot02 from "./xdsec-mascot-02.webp";
import xdsecMascot03 from "./xdsec-mascot-03.webp";
import xdsecMascot04 from "./xdsec-mascot-04-1.webp";
import xdsecMascot05 from "./xdsec-mascot-04-4.webp";
import xdsecMascotCrying from "./xdsec-mascot-crying.webp";
import xdsecMascotHappy from "./xdsec-mascot-happy.webp";
import xdsecMascotNormal from "./xdsec-mascot-normal.webp";
import xdsecMascotConfused from "./xdsec-mascot-question.webp";
import xdsecMascotUnsee from "./xdsec-mascot-unsee.webp";

export type Sticker = {
  src: string;
  alt: string;
};

export const stickerSet: Sticker[] = [
  { src: xdsecMascot01, alt: "Sigh" },
  { src: xdsecMascot02, alt: "Angry" },
  { src: xdsecMascot03, alt: "Eating" },
  { src: xdsecMascot04, alt: "Want peach" },
  { src: xdsecMascot05, alt: "Awake" },
  { src: xdsecMascotCrying, alt: "Crying" },
  { src: xdsecMascotHappy, alt: "Happy" },
  { src: xdsecMascotNormal, alt: "Stare" },
  { src: xdsecMascotUnsee, alt: "Wink" },
  { src: xdsecMascotConfused, alt: "Confused" },
];
