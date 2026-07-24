import { useEffect } from 'react';
import { CallEmbed } from '../plugins/call';
import { useSetting } from '../state/hooks/settings';
import { settingsAtom } from '../state/settings';

const isTypingTarget = (target: EventTarget | null): boolean => {
  if (!(target instanceof HTMLElement)) return false;
  return target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable;
};

const setMicrophone = (embed: CallEmbed, enabled: boolean) => {
  if (embed.control.microphone !== enabled) {
    embed.control.toggleMicrophone();
  }
};

export const usePushToTalk = (embed: CallEmbed, joined: boolean): void => {
  const [pushToTalk] = useSetting(settingsAtom, 'pushToTalk');
  const [pushToTalkKey] = useSetting(settingsAtom, 'pushToTalkKey');

  useEffect(() => {
    if (!pushToTalk || !joined) return undefined;

    let talking = false;
    setMicrophone(embed, false);

    const handleKeyDown = (evt: KeyboardEvent) => {
      if (evt.code !== pushToTalkKey || evt.repeat || isTypingTarget(evt.target)) return;
      talking = true;
      setMicrophone(embed, true);
    };
    const handleKeyUp = (evt: KeyboardEvent) => {
      if (evt.code !== pushToTalkKey || !talking) return;
      talking = false;
      setMicrophone(embed, false);
    };
    const handleBlur = () => {
      if (!talking) return;
      talking = false;
      setMicrophone(embed, false);
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    window.addEventListener('blur', handleBlur);
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
      window.removeEventListener('blur', handleBlur);
    };
  }, [pushToTalk, pushToTalkKey, embed, joined]);
};
