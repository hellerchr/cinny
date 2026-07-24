import { atom } from 'jotai';
import { CallEmbed } from '../plugins/call';

const baseCallFullscreenAtom = atom<boolean>(false);

export const callFullscreenAtom = atom<boolean, [boolean], void>(
  (get) => get(baseCallFullscreenAtom),
  (get, set, fullscreen) => {
    set(baseCallFullscreenAtom, fullscreen);
  }
);

const baseCallEmbedAtom = atom<CallEmbed | undefined>(undefined);

export const callEmbedAtom = atom<CallEmbed | undefined, [CallEmbed | undefined], void>(
  (get) => get(baseCallEmbedAtom),
  (get, set, callEmbed) => {
    const prevCallEmbed = get(baseCallEmbedAtom);
    if (callEmbed === prevCallEmbed) return;

    if (prevCallEmbed) {
      prevCallEmbed.dispose();
    }

    set(baseCallFullscreenAtom, false);
    set(baseCallEmbedAtom, callEmbed);
  }
);

export const callChatAtom = atom<boolean>(false);
