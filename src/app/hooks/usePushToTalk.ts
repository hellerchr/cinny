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

const isTauri = (): boolean =>
  typeof (window as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__ !== 'undefined';

const MODIFIER_CODES = [
  'MetaLeft',
  'MetaRight',
  'ShiftLeft',
  'ShiftRight',
  'ControlLeft',
  'ControlRight',
  'AltLeft',
  'AltRight',
];
const isModifierCode = (code: string): boolean => MODIFIER_CODES.includes(code);

// Global hotkey plugin cannot register modifier-only keys,
// so those are handled by the native event listener (see src-tauri/src/ptt.rs).
const registerNativeModifierKey = (key: string, onTalkingChange: (talking: boolean) => void) => {
  const unlisteners: Array<Promise<() => void>> = [];

  Promise.all([import('@tauri-apps/api/core'), import('@tauri-apps/api/event')])
    .then(([{ invoke }, { listen }]) => {
      invoke('set_ptt_key', { key }).catch(() => undefined);
      unlisteners.push(listen('ptt-key-press', () => onTalkingChange(true)));
      unlisteners.push(listen('ptt-key-release', () => onTalkingChange(false)));
    })
    .catch(() => undefined);

  return () => {
    import('@tauri-apps/api/core')
      .then(({ invoke }) => invoke('set_ptt_key', { key: null }))
      .catch(() => undefined);
    unlisteners.forEach((unlisten) => unlisten.then((fn) => fn()).catch(() => undefined));
  };
};

const registerGlobalShortcut = (key: string, onTalkingChange: (talking: boolean) => void) => {
  let unregister: (() => void) | undefined;
  let disposed = false;

  import('@tauri-apps/plugin-global-shortcut')
    .then((gs) => {
      if (disposed) return undefined;
      return gs
        .register(key, (event) => onTalkingChange(event.state === 'Pressed'))
        .then(() => {
          unregister = () => {
            gs.unregister(key).catch(() => undefined);
          };
        });
    })
    .catch(() => undefined);

  return () => {
    disposed = true;
    unregister?.();
  };
};

const registerGlobalKey = (key: string, onTalkingChange: (talking: boolean) => void) => {
  if (isModifierCode(key)) return registerNativeModifierKey(key, onTalkingChange);
  return registerGlobalShortcut(key, onTalkingChange);
};

export const usePushToTalk = (embed: CallEmbed, joined: boolean): void => {
  const [pushToTalk] = useSetting(settingsAtom, 'pushToTalk');
  const [pushToTalkKey] = useSetting(settingsAtom, 'pushToTalkKey');

  useEffect(() => {
    if (!pushToTalk || !joined) return undefined;

    let talking = false;
    const setTalking = (value: boolean) => {
      if (talking === value) return;
      talking = value;
      setMicrophone(embed, value);
    };

    setMicrophone(embed, false);

    const handleKeyDown = (evt: KeyboardEvent) => {
      if (evt.code !== pushToTalkKey || evt.repeat || isTypingTarget(evt.target)) return;
      setTalking(true);
    };
    const handleKeyUp = (evt: KeyboardEvent) => {
      if (evt.code !== pushToTalkKey) return;
      setTalking(false);
    };
    const handleBlur = () => setTalking(false);

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
    window.addEventListener('blur', handleBlur);

    const unregisterGlobal = isTauri() ? registerGlobalKey(pushToTalkKey, setTalking) : undefined;

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('keyup', handleKeyUp);
      window.removeEventListener('blur', handleBlur);
      unregisterGlobal?.();
    };
  }, [pushToTalk, pushToTalkKey, embed, joined]);
};
