import hertaMascotSpining from "./herta-spinning.gif";
import pangbaiMascotAfraid from "./pangbai-afraid.png";
import pangbaiMascotAmaze from "./pangbai-amaze.png";
import pangbaiMascotAngry from "./pangbai-angry.png";
import pangbaiMascotCool from "./pangbai-cool.png";
import pangbaiMascotCrying from "./pangbai-crying.png";
import pangbaiMascotCute from "./pangbai-cute.png";
import pangbaiMascotDaze from "./pangbai-daze.png";
import pangbaiMascotDislike from "./pangbai-dislike.png";
import pangbaiMascotLike from "./pangbai-like.png";
import pangbaiMascotLoading from "./pangbai-loading.png";
import pangbaiMascotPoor from "./pangbai-poor.png";
import pangbaiQuestion from "./pangbai-question.png";
import pangbaiMascotShy from "./pangbai-shy.png";
import pangbaiMascotSilent from "./pangbai-silent.png";
import pangbaiMascotSleep from "./pangbai-sleep.png";
import pangbaiMascotSpeechless from "./pangbai-speechless.png";

export type Sticker = {
  src: string;
  alt: string;
};

export const stickerSet: Sticker[] = [
  { src: hertaMascotSpining, alt: "转圈圈" },
  { src: pangbaiMascotPoor, alt: "可怜" },
  { src: pangbaiMascotLike, alt: "爱心" },
  { src: pangbaiMascotAmaze, alt: "惊喜" },
  { src: pangbaiMascotDaze, alt: "发呆" },
  { src: pangbaiMascotCute, alt: "卖萌" },
  { src: pangbaiMascotLoading, alt: "宕机中" },
  { src: pangbaiQuestion, alt: "疑问" },
  { src: pangbaiMascotCrying, alt: "哭" },
  { src: pangbaiMascotCool, alt: "酷" },
  { src: pangbaiMascotShy, alt: "害羞" },
  { src: pangbaiMascotAfraid, alt: "慌" },
  { src: pangbaiMascotSleep, alt: "睡觉" },
  { src: pangbaiMascotDislike, alt: "嫌弃" },
  { src: pangbaiMascotSilent, alt: "沉默" },
  { src: pangbaiMascotSpeechless, alt: "无语" },
  { src: pangbaiMascotAngry, alt: "生气" },
];
