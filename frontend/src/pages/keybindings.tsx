import { Button } from "@/components/ui/button";
import { Keyboard, Plus, X, RotateCcw } from "lucide-react";
import { useEffect, useState, useCallback } from "react";
import { toast } from "sonner";
import { invoke } from '@tauri-apps/api/core';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";

interface KeyBinding {
    key: string;
    modifiers?: string[];
}

interface KeybindingsConfig {
    toggle_input_mode: KeyBinding[];
    set_kana_mode: KeyBinding[];
    set_latin_mode: KeyBinding[];
    convert_hiragana: KeyBinding[];
    convert_katakana: KeyBinding[];
    convert_half_katakana: KeyBinding[];
    convert_full_latin: KeyBinding[];
    convert_half_latin: KeyBinding[];
}

const defaultKeybindings: KeybindingsConfig = {
    toggle_input_mode: [{ key: "Zenkaku/Hankaku" }],
    set_kana_mode: [],
    set_latin_mode: [],
    convert_hiragana: [{ key: "F6" }],
    convert_katakana: [{ key: "F7" }],
    convert_half_katakana: [{ key: "F8" }],
    convert_full_latin: [{ key: "F9" }],
    convert_half_latin: [{ key: "F10" }],
};

// 仮想キーコードからキー名へのマッピング
const keyCodeToName: { [key: number]: string } = {
    0x70: "F1", 0x71: "F2", 0x72: "F3", 0x73: "F4",
    0x74: "F5", 0x75: "F6", 0x76: "F7", 0x77: "F8",
    0x78: "F9", 0x79: "F10", 0x7A: "F11", 0x7B: "F12",
    0x7C: "F13", 0x7D: "F14", 0x7E: "F15", 0x7F: "F16",
    0x80: "F17", 0x81: "F18", 0x82: "F19", 0x83: "F20",
    0x84: "F21", 0x85: "F22", 0x86: "F23", 0x87: "F24",
    0xF3: "Zenkaku/Hankaku", 0xF4: "Zenkaku/Hankaku",
    0x1C: "Henkan", 0x1D: "Muhenkan", 0x15: "Hiragana",
};

interface ActionConfig {
    id: keyof KeybindingsConfig;
    label: string;
    description: string;
}

const actions: ActionConfig[] = [
    { id: "toggle_input_mode", label: "入力モード切り替え", description: "かな/英数をトグル" },
    { id: "set_kana_mode", label: "かなモードに切り替え", description: "かなモードに直接切り替え" },
    { id: "set_latin_mode", label: "英数モードに切り替え", description: "英数モードに直接切り替え" },
    { id: "convert_hiragana", label: "ひらがな変換", description: "入力をひらがなに変換" },
    { id: "convert_katakana", label: "カタカナ変換", description: "入力をカタカナに変換" },
    { id: "convert_half_katakana", label: "半角カタカナ変換", description: "入力を半角カタカナに変換" },
    { id: "convert_full_latin", label: "全角英数変換", description: "入力を全角英数に変換" },
    { id: "convert_half_latin", label: "半角英数変換", description: "入力を半角英数に変換" },
];

export const Keybindings = () => {
    const [config, setConfig] = useState<KeybindingsConfig>(defaultKeybindings);
    const [dialogOpen, setDialogOpen] = useState(false);
    const [currentAction, setCurrentAction] = useState<keyof KeybindingsConfig | null>(null);
    const [capturedKey, setCapturedKey] = useState<string | null>(null);

    useEffect(() => {
        invoke<any>("get_config")
            .then((data) => {
                if (data.keybindings) {
                    setConfig({
                        toggle_input_mode: data.keybindings.toggle_input_mode || defaultKeybindings.toggle_input_mode,
                        set_kana_mode: data.keybindings.set_kana_mode || [],
                        set_latin_mode: data.keybindings.set_latin_mode || [],
                        convert_hiragana: data.keybindings.convert_hiragana || defaultKeybindings.convert_hiragana,
                        convert_katakana: data.keybindings.convert_katakana || defaultKeybindings.convert_katakana,
                        convert_half_katakana: data.keybindings.convert_half_katakana || defaultKeybindings.convert_half_katakana,
                        convert_full_latin: data.keybindings.convert_full_latin || defaultKeybindings.convert_full_latin,
                        convert_half_latin: data.keybindings.convert_half_latin || defaultKeybindings.convert_half_latin,
                    });
                }
            })
            .catch(() => {
                // Keep default values if config fetch fails
            });
    }, []);

    const updateConfig = async (newKeybindings: KeybindingsConfig) => {
        try {
            const data = await invoke<any>("get_config");
            data.keybindings = newKeybindings;
            await invoke("update_config", { newConfig: data });
            setConfig(newKeybindings);
            return true;
        } catch (error) {
            toast.error("設定の更新に失敗しました");
            return false;
        }
    };

    const handleKeyCapture = useCallback((e: KeyboardEvent) => {
        e.preventDefault();
        e.stopPropagation();

        const keyCode = e.keyCode;
        const keyName = keyCodeToName[keyCode];

        if (keyName) {
            setCapturedKey(keyName);
        } else if (e.key.length === 1) {
            // 通常のキー
            setCapturedKey(e.key.toUpperCase());
        } else {
            setCapturedKey(e.key);
        }
    }, []);

    useEffect(() => {
        if (dialogOpen) {
            window.addEventListener("keydown", handleKeyCapture);
            return () => window.removeEventListener("keydown", handleKeyCapture);
        }
    }, [dialogOpen, handleKeyCapture]);

    const openKeyDialog = (action: keyof KeybindingsConfig) => {
        setCurrentAction(action);
        setCapturedKey(null);
        setDialogOpen(true);
    };

    const addKeyBinding = async () => {
        if (!currentAction || !capturedKey) return;

        const newConfig = { ...config };
        const existing = newConfig[currentAction].find(kb => kb.key === capturedKey);
        if (!existing) {
            newConfig[currentAction] = [...newConfig[currentAction], { key: capturedKey }];
            const success = await updateConfig(newConfig);
            if (success) {
                toast.success(`${capturedKey} を追加しました`, {
                    description: "変更を完全に適用するには、PCを再起動してください",
                    duration: 10000,
                });
            }
        } else {
            toast.info("このキーは既に設定されています");
        }

        setDialogOpen(false);
        setCapturedKey(null);
        setCurrentAction(null);
    };

    const removeKeyBinding = async (action: keyof KeybindingsConfig, index: number) => {
        const newConfig = { ...config };
        newConfig[action] = newConfig[action].filter((_, i) => i !== index);
        const success = await updateConfig(newConfig);
        if (success) {
            toast.success("キーバインドを削除しました", {
                description: "変更を完全に適用するには、PCを再起動してください",
                duration: 10000,
            });
        }
    };

    const resetToDefaults = async () => {
        const success = await updateConfig(defaultKeybindings);
        if (success) {
            toast.success("デフォルト設定に戻しました", {
                description: "変更を完全に適用するには、PCを再起動してください",
                duration: 10000,
            });
        }
    };

    return (
        <div className="space-y-8">
            <section className="space-y-2">
                <div className="flex items-center justify-between">
                    <h1 className="text-sm font-bold text-foreground">キーバインド設定</h1>
                    <Button variant="outline" size="sm" onClick={resetToDefaults}>
                        <RotateCcw className="h-4 w-4 mr-2" />
                        デフォルトに戻す
                    </Button>
                </div>
                <p className="text-xs text-muted-foreground">
                    入力モード切り替えやファンクションキーの割り当てをカスタマイズできます
                </p>
            </section>

            <section className="space-y-4">
                <h2 className="text-sm font-bold text-foreground">モード切り替え</h2>
                {actions.slice(0, 3).map((action) => (
                    <KeyBindingRow
                        key={action.id}
                        action={action}
                        bindings={config[action.id]}
                        onAdd={() => openKeyDialog(action.id)}
                        onRemove={(index) => removeKeyBinding(action.id, index)}
                    />
                ))}
            </section>

            <section className="space-y-4">
                <h2 className="text-sm font-bold text-foreground">変換キー</h2>
                {actions.slice(3).map((action) => (
                    <KeyBindingRow
                        key={action.id}
                        action={action}
                        bindings={config[action.id]}
                        onAdd={() => openKeyDialog(action.id)}
                        onRemove={(index) => removeKeyBinding(action.id, index)}
                    />
                ))}
            </section>

            <section className="space-y-2">
                <div className="rounded-md border p-4 bg-muted/50">
                    <p className="text-xs text-muted-foreground">
                        💡 F13〜F24キーはMac用キーボードや一部のカスタムキーボードで使用できます。
                        キーの追加後、変更を適用するにはPCを再起動してください。
                    </p>
                </div>
            </section>

            <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
                <DialogContent className="sm:max-w-md">
                    <DialogHeader>
                        <DialogTitle>キーを押してください</DialogTitle>
                        <DialogDescription>
                            割り当てたいキーを押してください
                        </DialogDescription>
                    </DialogHeader>
                    <div className="flex items-center justify-center py-8">
                        <div className="text-center">
                            <Keyboard className="h-12 w-12 mx-auto mb-4 text-muted-foreground" />
                            {capturedKey ? (
                                <div className="text-2xl font-bold">{capturedKey}</div>
                            ) : (
                                <div className="text-muted-foreground">キー入力待機中...</div>
                            )}
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setDialogOpen(false)}>
                            キャンセル
                        </Button>
                        <Button onClick={addKeyBinding} disabled={!capturedKey}>
                            設定する
                        </Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
};

interface KeyBindingRowProps {
    action: ActionConfig;
    bindings: KeyBinding[];
    onAdd: () => void;
    onRemove: (index: number) => void;
}

const KeyBindingRow = ({ action, bindings, onAdd, onRemove }: KeyBindingRowProps) => {
    return (
        <div className="rounded-md border p-4">
            <div className="flex items-start justify-between">
                <div className="space-y-1">
                    <p className="text-sm font-medium">{action.label}</p>
                    <p className="text-xs text-muted-foreground">{action.description}</p>
                </div>
            </div>
            <div className="mt-3 flex flex-wrap gap-2 items-center">
                {bindings.map((binding, index) => (
                    <div
                        key={index}
                        className="flex items-center gap-1 px-3 py-1.5 bg-secondary rounded-md text-sm"
                    >
                        <Keyboard className="h-3 w-3" />
                        <span>{binding.key}</span>
                        <button
                            onClick={() => onRemove(index)}
                            className="ml-1 hover:text-destructive"
                        >
                            <X className="h-3 w-3" />
                        </button>
                    </div>
                ))}
                <Button variant="outline" size="sm" onClick={onAdd}>
                    <Plus className="h-4 w-4 mr-1" />
                    キーを追加
                </Button>
            </div>
        </div>
    );
};

