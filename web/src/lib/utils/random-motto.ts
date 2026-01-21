const verbs = [
  "building",
  "creating",
  "developing",
  "designing",
  "implementing",
  "optimizing",
  "refactoring",
  "debugging",
  "deploying",
  "testing",
  "monitoring",
  "scaling",
  "exploring",
  "discovering",
  "learning",
  "mastering",
  "crafting",
  "solving",
  "connecting",
  "streaming",
  "processing",
  "analyzing",
  "integrating",
  "automating",
];

const adjectives = [
  "efficient",
  "robust",
  "scalable",
  "modern",
  "elegant",
  "innovative",
  "creative",
  "responsive",
  "dynamic",
  "flexible",
  "secure",
  "lightweight",
  "powerful",
  "smart",
  "intuitive",
  "seamless",
  "reliable",
  "advanced",
  "cutting_edge",
  "user_friendly",
  "high_performance",
  "cloud_native",
  "real_time",
  "cross_platform",
];

// technical nouns
const techNouns = [
  "API",
  "database",
  "framework",
  "library",
  "component",
  "service",
  "module",
  "pipeline",
  "workflow",
  "architecture",
  "infrastructure",
  "platform",
  "solution",
  "algorithm",
  "protocol",
  "interface",
  "dashboard",
  "analytics",
  "microservice",
  "container",
  "cluster",
  "network",
  "authentication",
  "middleware",
  "cache",
];

// life nouns
const lifeNouns = [
  "morning_routine",
  "coffee_break",
  "weekend_project",
  "garden_space",
  "kitchen_setup",
  "reading_corner",
  "workout_session",
  "movie_night",
  "dinner_party",
  "beach_walk",
  "hiking_trail",
  "city_exploration",
  "art_gallery",
  "music_festival",
  "book_club",
  "cooking_experiment",
  "travel_adventure",
  "sunset_viewing",
  "stargazing_session",
];

// contexts/domains
const contexts = [
  "development",
  "production",
  "testing",
  "staging",
  "deployment",
  "monitoring",
  "performance",
  "security",
  "scalability",
  "maintenance",
  "integration",
  "automation",
  "experience",
  "lifestyle",
  "wellness",
  "creativity",
  "productivity",
  "entertainment",
  "education",
  "collaboration",
  "innovation",
  "exploration",
  "relaxation",
  "adventure",
];

// connectors and prepositions
const connectors = ["with", "for", "in", "on", "through", "using", "via", "across", "within"];

export function generateRandomMotto() {
  function getRandomItem<T>(array: T[]): T {
    return array[Math.floor(Math.random() * array.length)];
  }
  const patterns = [
    // technical patterns
    () => `${getRandomItem(verbs)}_${getRandomItem(adjectives)}_${getRandomItem(techNouns)}`,
    () =>
      `${getRandomItem(adjectives)}_${getRandomItem(techNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
    () => `${getRandomItem(verbs)}_${getRandomItem(techNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
    () => `${getRandomItem(adjectives)}_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_solution`,

    // life patterns
    () => `${getRandomItem(verbs)}_${getRandomItem(adjectives)}_${getRandomItem(lifeNouns)}`,
    () =>
      `${getRandomItem(adjectives)}_${getRandomItem(lifeNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
    () => `${getRandomItem(verbs)}_perfect_${getRandomItem(lifeNouns)}_experience`,
    () => `daily_${getRandomItem(verbs)}_${getRandomItem(lifeNouns)}_routine`,

    // mixed patterns
    () => `${getRandomItem(verbs)}_${getRandomItem(adjectives)}_${getRandomItem(contexts)}_platform`,
    () =>
      `smart_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_${getRandomItem(connectors)}_${getRandomItem(contexts)}`,
    () => `${getRandomItem(adjectives)}_${getRandomItem(verbs)}_${getRandomItem(contexts)}_workflow`,
    () => `next_gen_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_system`,

    // more complex patterns
    () => `${getRandomItem(verbs)}_and_${getRandomItem(verbs)}_${getRandomItem(techNouns)}`,
    () => `${getRandomItem(adjectives)}_${getRandomItem(lifeNouns)}_${getRandomItem(connectors)}_modern_life`,
    () =>
      `${getRandomItem(verbs)}_${getRandomItem(contexts)}_${getRandomItem(connectors)}_${getRandomItem(adjectives)}_way`,
    () => `auto_${getRandomItem(verbs)}_${getRandomItem(techNouns)}_${getRandomItem(contexts)}`,
  ];

  const pattern = getRandomItem(patterns);
  return pattern();
}
