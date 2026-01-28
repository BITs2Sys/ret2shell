const gitmojiMap: Record<string, string> = {
  ":sparkles:": "icon-[fluent-emoji-flat--sparkles]",
  ":building_construction:": "icon-[fluent-emoji-flat--building-construction]",
  ":package:": "icon-[fluent-emoji-flat--package]",
  ":fire:": "icon-[fluent-emoji-flat--fire]",
  ":speech_balloon:": "icon-[fluent-emoji-flat--speech-balloon]",
  ":memo:": "icon-[fluent-emoji-flat--memo]",
  ":tada:": "icon-[fluent-emoji-flat--party-popper]",
  ":arrow_up:": "icon-[fluent-emoji-flat--up-arrow]",
};

export function transformGitmoji(message: string): { icon?: string; text: string } {
  for (const [gitmoji, icon] of Object.entries(gitmojiMap)) {
    if (message.startsWith(gitmoji)) {
      return {
        icon,
        text: message.slice(gitmoji.length).trimStart(),
      };
    }
  }
  return { text: message };
}
