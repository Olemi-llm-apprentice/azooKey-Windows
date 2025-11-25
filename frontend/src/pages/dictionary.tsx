import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
} from "@/components/ui/table";
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
    DialogTrigger,
} from "@/components/ui/dialog";
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { Book, Plus, Pencil, Trash2, Download, Upload, Search } from "lucide-react";
import { useEffect, useState } from "react";
import { toast } from "sonner";
import { invoke } from '@tauri-apps/api/core';
import { save, open } from '@tauri-apps/plugin-dialog';

interface DictEntry {
    reading: string;
    word: string;
    part_of_speech: string;
}

const PART_OF_SPEECH_OPTIONS = [
    "普通名詞",
    "固有名詞",
    "人名",
    "地名",
    "動詞",
    "形容詞",
    "その他",
];

export const Dictionary = () => {
    const [entries, setEntries] = useState<DictEntry[]>([]);
    const [searchQuery, setSearchQuery] = useState("");
    const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
    const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
    const [editIndex, setEditIndex] = useState<number | null>(null);
    const [formData, setFormData] = useState<DictEntry>({
        reading: "",
        word: "",
        part_of_speech: "その他",
    });

    // 辞書データの読み込み
    const loadDictionary = async () => {
        try {
            const data = await invoke<{ entries: DictEntry[] }>("get_user_dictionary");
            setEntries(data.entries);
        } catch (error) {
            toast.error("辞書の読み込みに失敗しました");
        }
    };

    useEffect(() => {
        loadDictionary();
    }, []);

    // フィルタリングされたエントリ
    const filteredEntries = entries.filter(
        (entry) =>
            entry.reading.includes(searchQuery) ||
            entry.word.includes(searchQuery)
    );

    // 単語追加
    const handleAdd = async () => {
        try {
            await invoke("add_dictionary_entry", { entry: formData });
            toast.success("単語を追加しました");
            setIsAddDialogOpen(false);
            setFormData({ reading: "", word: "", part_of_speech: "その他" });
            loadDictionary();
        } catch (error) {
            toast.error(error as string);
        }
    };

    // 単語編集
    const handleEdit = async () => {
        if (editIndex === null) return;
        try {
            await invoke("update_dictionary_entry", { index: editIndex, entry: formData });
            toast.success("単語を更新しました");
            setIsEditDialogOpen(false);
            setEditIndex(null);
            setFormData({ reading: "", word: "", part_of_speech: "その他" });
            loadDictionary();
        } catch (error) {
            toast.error(error as string);
        }
    };

    // 単語削除
    const handleDelete = async (index: number) => {
        try {
            await invoke("remove_dictionary_entry", { index });
            toast.success("単語を削除しました");
            loadDictionary();
        } catch (error) {
            toast.error("削除に失敗しました");
        }
    };

    // エクスポート
    const handleExport = async () => {
        try {
            const filePath = await save({
                filters: [{ name: "TSV", extensions: ["txt", "tsv"] }],
                defaultPath: "user_dict.txt",
            });
            if (filePath) {
                await invoke("export_dictionary", { path: filePath });
                toast.success("辞書をエクスポートしました");
            }
        } catch (error) {
            toast.error("エクスポートに失敗しました");
        }
    };

    // インポート
    const handleImport = async (merge: boolean) => {
        try {
            const filePath = await open({
                filters: [{ name: "TSV", extensions: ["txt", "tsv"] }],
                multiple: false,
            });
            if (filePath) {
                const count = await invoke<number>("import_dictionary", { path: filePath, merge });
                toast.success(`${count}件の単語をインポートしました`);
                loadDictionary();
            }
        } catch (error) {
            toast.error("インポートに失敗しました");
        }
    };

    // 編集ダイアログを開く
    const openEditDialog = (index: number) => {
        setEditIndex(index);
        setFormData(entries[index]);
        setIsEditDialogOpen(true);
    };

    return (
        <div className="space-y-6">
            <section className="space-y-2">
                <h1 className="text-sm font-bold text-foreground">ユーザー辞書</h1>
                <p className="text-xs text-muted-foreground">
                    独自の単語を登録して、変換候補に表示させることができます
                </p>
            </section>

            {/* ツールバー */}
            <div className="flex items-center justify-between gap-4">
                <div className="relative flex-1 max-w-sm">
                    <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
                    <Input
                        placeholder="検索..."
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        className="pl-8"
                    />
                </div>
                <div className="flex items-center gap-2">
                    <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
                        <DialogTrigger asChild>
                            <Button size="sm">
                                <Plus className="h-4 w-4 mr-1" />
                                追加
                            </Button>
                        </DialogTrigger>
                        <DialogContent>
                            <DialogHeader>
                                <DialogTitle>単語を追加</DialogTitle>
                                <DialogDescription>
                                    新しい単語を辞書に登録します
                                </DialogDescription>
                            </DialogHeader>
                            <div className="space-y-4 py-4">
                                <div className="space-y-2">
                                    <Label htmlFor="reading">読み（ひらがな）</Label>
                                    <Input
                                        id="reading"
                                        placeholder="あずーきー"
                                        value={formData.reading}
                                        onChange={(e) => setFormData({ ...formData, reading: e.target.value })}
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="word">単語</Label>
                                    <Input
                                        id="word"
                                        placeholder="azooKey"
                                        value={formData.word}
                                        onChange={(e) => setFormData({ ...formData, word: e.target.value })}
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label htmlFor="pos">品詞</Label>
                                    <Select
                                        value={formData.part_of_speech}
                                        onValueChange={(value) => setFormData({ ...formData, part_of_speech: value })}
                                    >
                                        <SelectTrigger>
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            {PART_OF_SPEECH_OPTIONS.map((pos) => (
                                                <SelectItem key={pos} value={pos}>
                                                    {pos}
                                                </SelectItem>
                                            ))}
                                        </SelectContent>
                                    </Select>
                                </div>
                            </div>
                            <DialogFooter>
                                <Button variant="outline" onClick={() => setIsAddDialogOpen(false)}>
                                    キャンセル
                                </Button>
                                <Button onClick={handleAdd}>追加</Button>
                            </DialogFooter>
                        </DialogContent>
                    </Dialog>

                    <Button variant="outline" size="sm" onClick={handleExport}>
                        <Download className="h-4 w-4 mr-1" />
                        エクスポート
                    </Button>

                    <AlertDialog>
                        <AlertDialogTrigger asChild>
                            <Button variant="outline" size="sm">
                                <Upload className="h-4 w-4 mr-1" />
                                インポート
                            </Button>
                        </AlertDialogTrigger>
                        <AlertDialogContent>
                            <AlertDialogHeader>
                                <AlertDialogTitle>辞書をインポート</AlertDialogTitle>
                                <AlertDialogDescription>
                                    インポート方法を選択してください
                                </AlertDialogDescription>
                            </AlertDialogHeader>
                            <AlertDialogFooter>
                                <AlertDialogCancel>キャンセル</AlertDialogCancel>
                                <AlertDialogAction onClick={() => handleImport(true)}>
                                    既存に追加
                                </AlertDialogAction>
                                <AlertDialogAction onClick={() => handleImport(false)}>
                                    上書き
                                </AlertDialogAction>
                            </AlertDialogFooter>
                        </AlertDialogContent>
                    </AlertDialog>
                </div>
            </div>

            {/* 辞書テーブル */}
            <div className="rounded-md border">
                <Table>
                    <TableHeader>
                        <TableRow>
                            <TableHead>読み</TableHead>
                            <TableHead>単語</TableHead>
                            <TableHead>品詞</TableHead>
                            <TableHead className="w-[100px]">操作</TableHead>
                        </TableRow>
                    </TableHeader>
                    <TableBody>
                        {filteredEntries.length === 0 ? (
                            <TableRow>
                                <TableCell colSpan={4} className="text-center text-muted-foreground py-8">
                                    <Book className="h-8 w-8 mx-auto mb-2 opacity-50" />
                                    <p>登録された単語がありません</p>
                                </TableCell>
                            </TableRow>
                        ) : (
                            filteredEntries.map((entry, index) => (
                                <TableRow key={index}>
                                    <TableCell className="font-mono">{entry.reading}</TableCell>
                                    <TableCell>{entry.word}</TableCell>
                                    <TableCell className="text-muted-foreground text-sm">
                                        {entry.part_of_speech}
                                    </TableCell>
                                    <TableCell>
                                        <div className="flex items-center gap-1">
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                className="h-8 w-8"
                                                onClick={() => openEditDialog(entries.indexOf(entry))}
                                            >
                                                <Pencil className="h-4 w-4" />
                                            </Button>
                                            <AlertDialog>
                                                <AlertDialogTrigger asChild>
                                                    <Button
                                                        variant="ghost"
                                                        size="icon"
                                                        className="h-8 w-8 text-destructive"
                                                    >
                                                        <Trash2 className="h-4 w-4" />
                                                    </Button>
                                                </AlertDialogTrigger>
                                                <AlertDialogContent>
                                                    <AlertDialogHeader>
                                                        <AlertDialogTitle>単語を削除しますか？</AlertDialogTitle>
                                                        <AlertDialogDescription>
                                                            「{entry.word}」を辞書から削除します。この操作は取り消せません。
                                                        </AlertDialogDescription>
                                                    </AlertDialogHeader>
                                                    <AlertDialogFooter>
                                                        <AlertDialogCancel>キャンセル</AlertDialogCancel>
                                                        <AlertDialogAction onClick={() => handleDelete(entries.indexOf(entry))}>
                                                            削除
                                                        </AlertDialogAction>
                                                    </AlertDialogFooter>
                                                </AlertDialogContent>
                                            </AlertDialog>
                                        </div>
                                    </TableCell>
                                </TableRow>
                            ))
                        )}
                    </TableBody>
                </Table>
            </div>

            {/* 統計 */}
            <p className="text-xs text-muted-foreground">
                登録単語数: {entries.length}件
                {searchQuery && ` (表示: ${filteredEntries.length}件)`}
            </p>

            {/* 編集ダイアログ */}
            <Dialog open={isEditDialogOpen} onOpenChange={setIsEditDialogOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>単語を編集</DialogTitle>
                        <DialogDescription>
                            単語の情報を編集します
                        </DialogDescription>
                    </DialogHeader>
                    <div className="space-y-4 py-4">
                        <div className="space-y-2">
                            <Label htmlFor="edit-reading">読み（ひらがな）</Label>
                            <Input
                                id="edit-reading"
                                value={formData.reading}
                                onChange={(e) => setFormData({ ...formData, reading: e.target.value })}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="edit-word">単語</Label>
                            <Input
                                id="edit-word"
                                value={formData.word}
                                onChange={(e) => setFormData({ ...formData, word: e.target.value })}
                            />
                        </div>
                        <div className="space-y-2">
                            <Label htmlFor="edit-pos">品詞</Label>
                            <Select
                                value={formData.part_of_speech}
                                onValueChange={(value) => setFormData({ ...formData, part_of_speech: value })}
                            >
                                <SelectTrigger>
                                    <SelectValue />
                                </SelectTrigger>
                                <SelectContent>
                                    {PART_OF_SPEECH_OPTIONS.map((pos) => (
                                        <SelectItem key={pos} value={pos}>
                                            {pos}
                                        </SelectItem>
                                    ))}
                                </SelectContent>
                            </Select>
                        </div>
                    </div>
                    <DialogFooter>
                        <Button variant="outline" onClick={() => setIsEditDialogOpen(false)}>
                            キャンセル
                        </Button>
                        <Button onClick={handleEdit}>保存</Button>
                    </DialogFooter>
                </DialogContent>
            </Dialog>
        </div>
    );
};

