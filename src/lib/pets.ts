export interface PetMeta {
  agentId: string;
  sprite: string;
  name: string;
  role: string;
}

export const PETS: PetMeta[] = [
  { agentId: "orchestrator", sprite: "micheal", name: "Micheal", role: "Orchestrator" },
  { agentId: "dreamer", sprite: "holly", name: "Holly", role: "Memory" },
  { agentId: "forge", sprite: "dwight", name: "Dwight", role: "Backend" },
  { agentId: "pixel", sprite: "jim", name: "Jim", role: "Frontend" },
  { agentId: "scout", sprite: "ryan", name: "Ryan", role: "Research" },
  { agentId: "voice", sprite: "andy", name: "Andy", role: "Marketing" },
  { agentId: "probe", sprite: "erin", name: "Erin", role: "Testing" },
  { agentId: "iris", sprite: "pam", name: "Pam", role: "UI/UX Designer" },
  { agentId: "verity", sprite: "stanley", name: "Stanley", role: "Auditing Engineer" },
];

export function petForAgent(agentId: string | null | undefined): PetMeta | undefined {
  return PETS.find((pet) => pet.agentId === agentId);
}
